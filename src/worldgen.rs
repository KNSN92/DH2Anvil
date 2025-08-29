use std::{
    collections::HashSet,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use fastanvil::Region;

use crate::{
    chunk::{AIR, BlockState, Chunk},
    data::{DH_SECTION_WIDTH, DHSectionData, DHSectionPos},
};

const Y_OFFSET: i32 = -64;

// section_pos / 8 = region_pos
const SECTION_REGION_SCALE: usize = 512 / DH_SECTION_WIDTH;

pub const CHUNK_TEMP: &[u8] = include_bytes!("../chunk.nbt");

pub struct DHData2WorldGenerator<F>
where
    F: Fn(&DHSectionPos) -> DHSectionData,
{
    out_dir: PathBuf,
    section_poses: HashSet<DHSectionPos>,
    request_section: F,
}

impl<F: Fn(&DHSectionPos) -> DHSectionData> DHData2WorldGenerator<F> {
    pub fn new(
        out_dir: impl AsRef<Path>,
        section_poses: Vec<DHSectionPos>,
        request_section: F,
    ) -> Result<Self> {
        if !out_dir.as_ref().is_dir() {
            bail!(
                "{} is not a directory",
                out_dir.as_ref().to_str().unwrap_or("None")
            );
        }
        Ok(DHData2WorldGenerator {
            out_dir: out_dir.as_ref().to_path_buf(),
            section_poses: section_poses.into_iter().collect(),
            request_section,
        })
    }

    pub fn generate(&mut self, on_write_section: impl Fn(DHSectionPos, u64)) -> Result<()> {
        let chunk_temp = fastnbt::from_bytes::<Chunk>(CHUNK_TEMP)?;
        let mut total_size = 0u64;
        while self.section_poses.len() > 0 {
            let section_pos = self.section_poses.iter().next().unwrap();
            let (region_x, region_z) = (section_pos.x >> 3, section_pos.z >> 3);
            let (region_snapped_section_x, region_snapped_section_z) =
                (region_x << 3, region_z << 3);

            let region = self.out_dir.join(format!("r.{region_x}.{region_z}.mca"));
            let region_file = File::options()
                .read(true)
                .write(true)
                .create(true)
                .open(region)?;
            let mut region = Region::create(&region_file)?;
            let mut estimated_region_size = 0;
            for region_oriented_section_x in 0..SECTION_REGION_SCALE {
                for region_oriented_section_z in 0..SECTION_REGION_SCALE {
                    let section_pos = DHSectionPos {
                        x: region_snapped_section_x + region_oriented_section_x as i32,
                        z: region_snapped_section_z + region_oriented_section_z as i32,
                    };
                    if !self.section_poses.contains(&section_pos) {
                        continue;
                    }
                    self.section_poses.remove(&section_pos);
                    let dh_section = (self.request_section)(&section_pos);
                    // Chunks in current section
                    let mut chunks = Vec::with_capacity(16);
                    for i in 0..16 {
                        let mut chunk = chunk_temp.clone();
                        chunk.set_chunk_pos(
                            &(section_pos.x) * 4 + (i >> 2) as i32,
                            &(section_pos.z) * 4 + (i & 3) as i32,
                        );
                        chunk.set_biome("minecraft:plains".to_string());
                        chunk.set_status("minecraft:initialize_light".to_string());
                        // chunk.light();
                        chunks.push(chunk);
                    }
                    for x in 0..DH_SECTION_WIDTH {
                        for z in 0..DH_SECTION_WIDTH {
                            let chunk = &mut chunks[(x & 0x30) >> 2 | (z & 0x30) >> 4];
                            let data_points = &dh_section.data[x * DH_SECTION_WIDTH + z];
                            for data_point in data_points {
                                let mapping = &dh_section.mapping[data_point.id as usize];
                                let block = mapping.block.clone();
                                let state = &mapping.block_state;
                                let block = BlockState {
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
                                };
                                for y in data_point.min_y..data_point.min_y + data_point.height {
                                    chunk.set_block(
                                        x as u32 & 15,
                                        (y + Y_OFFSET).min(319),
                                        z as u32 & 15,
                                        block.clone(),
                                    )?;
                                }
                            }
                        }
                    }
                    for i in 0..16 {
                        let chunk = &chunks[i];
                        let chunk = &fastnbt::to_bytes(&chunk)?;
                        estimated_region_size += chunk.len() as u64;
                        region.write_chunk(
                            (region_oriented_section_x * 4 + (i >> 2)) & 511,
                            (region_oriented_section_z * 4 + (i & 3)) & 511,
                            chunk,
                        )?;
                    }
                    (on_write_section)(section_pos, total_size + estimated_region_size);
                }
            }
            total_size += region_file.metadata()?.len();
        }
        Ok(())
    }
}
