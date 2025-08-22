use std::{collections::HashSet, io::Cursor};

use anyhow::Result;
use fastanvil::{CurrentJavaChunk, JavaChunk, Region};

use crate::data::{DH_SECTION_WIDTH, DHSectionData, DHSectionPos};

const Y_OFFSET: i32 = -64;

// section_pos / 8 = region_pos
const SECTION_REGION_SCALE: usize = 512 / DH_SECTION_WIDTH;

pub struct DHData2WorldGenerator<F>
where
    F: Fn(DHSectionPos) -> DHSectionData,
{
    section_poses: HashSet<DHSectionPos>,
    request_section: F,
}

impl<F: Fn(DHSectionPos) -> DHSectionData> DHData2WorldGenerator<F> {
    pub fn new(section_poses: Vec<DHSectionPos>, request_section: F) -> Self {
        DHData2WorldGenerator {
            section_poses: section_poses.into_iter().collect(),
            request_section,
        }
    }

    pub fn generate(&mut self) -> Result<()> {
        while self.section_poses.len() > 0 {
            let section_pos = self.section_poses.iter().next().unwrap();
            let (region_x, region_z) = (
                section_pos.x / SECTION_REGION_SCALE as i32,
                section_pos.z / SECTION_REGION_SCALE as i32,
            );
            let (region_snapped_section_x, region_snapped_section_z) = (
                region_x * SECTION_REGION_SCALE as i32,
                region_z * SECTION_REGION_SCALE as i32,
            );
            let region_data = Cursor::new(Vec::<u8>::new());
            let region = Region::create(region_data)?;
            for region_oriented_section_x in 0..SECTION_REGION_SCALE {
                for region_oriented_section_z in 0..SECTION_REGION_SCALE {
                    let section_pos = DHSectionPos {
                        x: (region_oriented_section_x + region_oriented_section_x) as i32,
                        z: (region_oriented_section_z + region_oriented_section_z) as i32,
                    };
                    if !self.section_poses.contains(&section_pos) {
                        continue;
                    }
                    self.section_poses.remove(&section_pos);
                    let dh_section = (self.request_section)(section_pos);
                    // let palette = [];
                    for x in 0..DH_SECTION_WIDTH {
                        for z in 0..DH_SECTION_WIDTH {
                            let data_points = dh_section.data[x * DH_SECTION_WIDTH + z];
                            let (x, z) = (
                                region_oriented_section_x * DH_SECTION_WIDTH + x,
                                region_oriented_section_z * DH_SECTION_WIDTH + z,
                            );
                            let chunk = match region.wr(x << 4, z << 4)? {
                                Some(raw_chunk) => JavaChunk::from_bytes(&raw_chunk)?,
                                None => JavaChunk::Post18(CurrentJavaChunk {
                                    data_version: 0,
                                    status: String::from("full"),
                                    sections: None,
                                    heightmaps: None,
                                }),
                            };
                            for data_point in data_points {}
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
