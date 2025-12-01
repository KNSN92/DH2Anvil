use std::{
    collections::{HashMap, HashSet},
    io::{Read, Seek, Write},
    path::{Path, PathBuf},
    sync::{LazyLock, mpsc::Sender},
};

use anyhow::{Result, ensure};
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

static TEMP_CHUNK: LazyLock<Chunk> =
    LazyLock::new(|| fastnbt::from_bytes::<Chunk>(include_bytes!("../chunk.nbt")).unwrap());

pub enum WorldGenStatus {
    StartRegion {
        pos: RegionPos,
        thread_idx: Option<usize>,
        file_path: PathBuf,
    },
    FinishDHSection {
        pos: DHSectionPos,
    },
    FinishRegion {
        pos: RegionPos,
    },
    SkipRegion,
}

pub fn generate_world(
    region_poses: Vec<RegionPos>,
    section_requester: impl DHDataRequester + Send + Sync,
    out_dir: impl AsRef<Path>,
    overwrite_file: bool,
    status_sender: Sender<WorldGenStatus>,
) -> Result<()> {
    ensure!(
        out_dir.as_ref().is_dir(),
        "{} is not a directory",
        out_dir.as_ref().to_str().unwrap_or("None")
    );
    let out_dir = out_dir.as_ref().to_path_buf();
    let region_poses = region_poses
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    // region_poses.par_sort_by(|a, b| (a.x.abs() + a.z.abs()).cmp(&(b.x.abs() + b.z.abs())));
    region_poses
        .into_par_iter()
        .try_for_each(|region_pos| -> Result<()> {
            let region_file_path = out_dir.join(get_region_filename(&region_pos));
            let region_file = tempfile::NamedTempFile::new()?;
            status_sender.send(WorldGenStatus::StartRegion {
                pos: region_pos,
                thread_idx: rayon::current_thread_index(),
                file_path: region_file.path().to_path_buf(),
            })?;
            let dh_sections = section_requester.request_sections_in_region(&region_pos)?;
            generate_region(region_pos, dh_sections, &region_file, &status_sender)?;
            if overwrite_file || !region_file_path.exists() {
                region_file.persist(region_file_path)?;
                status_sender.send(WorldGenStatus::FinishRegion { pos: region_pos })?;
            } else {
                status_sender.send(WorldGenStatus::SkipRegion)?;
            }
            Ok(())
        })?;
    Ok(())
}

pub fn get_region_filename(pos: &RegionPos) -> String {
    format!("r.{}.{}.mca", pos.x, pos.z)
}

fn generate_region(
    region_pos: RegionPos,
    dh_sections: HashMap<DHSectionPos, DHSectionData>,
    stream: impl Read + Write + Seek,
    status_sender: &Sender<WorldGenStatus>,
) -> Result<()> {
    let region_snapped_section_pos = DHSectionPos::from(region_pos);

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
            let mut chunks = init_section_chunks(&section_pos);
            for x in 0..DH_SECTION_WIDTH {
                for z in 0..DH_SECTION_WIDTH {
                    let chunk = &mut chunks[(x & 0x30) >> 2 | (z & 0x30) >> 4];
                    let data_points = &dh_section.data[x * DH_SECTION_WIDTH + z];
                    for data_point in data_points {
                        let (block, biome) = get_block_biome(data_point, dh_section);
                        for y in data_point.min_y..data_point.min_y + data_point.height {
                            chunk.set_block_biome(
                                x as u32 & 0xf,
                                (y + Y_OFFSET).min(319),
                                z as u32 & 0xf,
                                block.clone(),
                                biome.clone(),
                            )?;
                        }
                    }
                }
            }
            for (i, chunk) in chunks.iter().enumerate().take(16) {
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

fn init_section_chunks(pos: &DHSectionPos) -> Vec<Chunk> {
    let mut chunks = Vec::with_capacity(16);
    for i in 0..16 {
        let mut chunk = TEMP_CHUNK.clone();
        chunk.set_chunk_pos(&(pos.x) * 4 + (i >> 2), &(pos.z) * 4 + (i & 3));
        chunk.set_status("minecraft:initialize_light".to_string());
        chunks.push(chunk);
    }
    chunks
}

fn get_block_biome(
    data_point: &DHFullDataPoint,
    dh_section: &DHSectionData,
) -> (BlockState, String) {
    let mapping = &dh_section.mapping[data_point.id as usize];
    let block = mapping.block.clone();
    let state = &mapping.block_state;
    let block_state = BlockState {
        name: block.unwrap_or_else(|| AIR.to_string()),
        properties: if !state.is_empty() {
            Some(state.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        } else {
            None
        },
    };
    let biome = mapping.biome.clone();
    (block_state, biome)
}
