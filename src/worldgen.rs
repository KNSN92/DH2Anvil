use std::{
    collections::{HashMap, HashSet},
    fs::{File, remove_file},
    io::{Read, Seek, Write},
    path::Path,
    sync::mpsc::Sender,
};

use anyhow::{Result, bail};
use fastanvil::Region;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    chunk::{AIR, BlockState, Chunk},
    data::{
        DH_SECTION_WIDTH, DHDataRequester, DHFullDataPoint, DHSectionData, DHSectionPos, RegionPos,
    },
};

const Y_OFFSET: i32 = -64;

// section_pos / 8 = region_pos
pub const SECTION_REGION_SCALE: usize = 512 / DH_SECTION_WIDTH;

const CHUNK_TEMP: &[u8] = include_bytes!("../chunk.nbt");

pub enum WorldGenStatus {
    StartRegion { pos: RegionPos, thread_idx: usize },
    FinishDHSection { pos: DHSectionPos },
    FinishRegion { pos: RegionPos },
}

pub fn generate(
    region_poses: Vec<RegionPos>,
    section_requester: impl DHDataRequester + Send + Sync,
    out_dir: impl AsRef<Path>,
    status_sender: Sender<WorldGenStatus>,
) -> Result<()> {
    if !out_dir.as_ref().is_dir() {
        bail!(
            "{} is not a directory",
            out_dir.as_ref().to_str().unwrap_or("None")
        );
    }
    let out_dir = out_dir.as_ref().to_path_buf();
    let temp_chunk = fastnbt::from_bytes::<Chunk>(CHUNK_TEMP)?;
    let region_poses = region_poses.into_iter().collect::<HashSet<_>>();
    region_poses
        .into_par_iter()
        .try_for_each(|region_pos| -> Result<()> {
            let dh_sections = section_requester.request_sections_in_region(&region_pos)?;
            if dh_sections.is_empty() {
                return Ok(());
            }
            let region_file = out_dir.join(format!("r.{}.{}.mca", region_pos.x, region_pos.z));
            if region_file.exists() {
                remove_file(&region_file)?;
            }
            let region_file = File::options()
                .read(true)
                .write(true)
                .create(true)
                .open(region_file)?;
            status_sender.send(WorldGenStatus::StartRegion {
                pos: region_pos,
                thread_idx: rayon::current_thread_index().unwrap(),
            })?;
            generate_region(
                &region_pos,
                dh_sections,
                &region_file,
                &temp_chunk,
                &status_sender,
            )?;
            status_sender.send(WorldGenStatus::FinishRegion { pos: region_pos })?;
            Result::Ok(())
        })?;
    Ok(())
}

fn generate_region(
    region_pos: &RegionPos,
    dh_sections: HashMap<DHSectionPos, DHSectionData>,
    stream: impl Read + Write + Seek,
    chunk_temp: &Chunk,
    status_sender: &Sender<WorldGenStatus>,
) -> Result<()> {
    let region_snapped_section_pos = DHSectionPos {
        x: region_pos.x << 3,
        z: region_pos.z << 3,
    };

    let mut region = Region::create(stream)?;
    for region_oriented_section_x in 0..SECTION_REGION_SCALE {
        for region_oriented_section_z in 0..SECTION_REGION_SCALE {
            let section_pos = DHSectionPos {
                x: region_snapped_section_pos.x + region_oriented_section_x as i32,
                z: region_snapped_section_pos.z + region_oriented_section_z as i32,
            };
            let dh_section = if let Some(dh_section) = dh_sections.get(&section_pos) {
                dh_section
            } else {
                status_sender.send(WorldGenStatus::FinishDHSection { pos: section_pos })?;
                continue;
            };
            // Chunks in current section
            let mut chunks = init_section_chunks(&chunk_temp, &section_pos);
            for x in 0..DH_SECTION_WIDTH {
                for z in 0..DH_SECTION_WIDTH {
                    let chunk = &mut chunks[(x & 0x30) >> 2 | (z & 0x30) >> 4];
                    let data_points = &dh_section.data[x * DH_SECTION_WIDTH + z];
                    for data_point in data_points {
                        let block = get_block(data_point, &dh_section);
                        for y in data_point.min_y..data_point.min_y + data_point.height {
                            chunk.set_block(
                                x as u32 & 0xf,
                                (y + Y_OFFSET).min(319),
                                z as u32 & 0xf,
                                block.clone(),
                            )?;
                        }
                    }
                }
            }
            for i in 0..16 {
                let chunk = &chunks[i];
                let chunk = &fastnbt::to_bytes(&chunk)?;
                region.write_chunk(
                    (region_oriented_section_x * 4 + (i >> 2)) & 0x1ff,
                    (region_oriented_section_z * 4 + (i & 3)) & 0x1ff,
                    chunk,
                )?;
            }
            status_sender.send(WorldGenStatus::FinishDHSection { pos: section_pos })?;
        }
    }
    Ok(())
}

fn init_section_chunks(chunk_temp: &Chunk, pos: &DHSectionPos) -> Vec<Chunk> {
    let mut chunks = Vec::with_capacity(16);
    for i in 0..16 {
        let mut chunk = chunk_temp.clone();
        chunk.set_chunk_pos(
            &(pos.x) * 4 + (i >> 2) as i32,
            &(pos.z) * 4 + (i & 3) as i32,
        );
        chunk.set_biome("minecraft:plains".to_string());
        chunk.set_status("minecraft:initialize_light".to_string());
        // chunk.light();
        chunks.push(chunk);
    }
    chunks
}

fn get_block(data_point: &DHFullDataPoint, dh_section: &DHSectionData) -> BlockState {
    let mapping = &dh_section.mapping[data_point.id as usize];
    let block = mapping.block.clone();
    let state = &mapping.block_state;
    BlockState {
        name: block.unwrap_or_else(|| AIR.to_string()),
        properties: if state.len() > 0 {
            Some(
                state
                    .into_iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            )
        } else {
            None
        },
    }
}
