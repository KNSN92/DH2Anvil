use std::{
    collections::HashMap,
    io::{Cursor, Read},
};

use anyhow::{Result, bail, ensure};
use byteorder::{BigEndian, ReadBytesExt};

use crate::decompress::CompressionMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegionPos {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DHSectionPos {
    pub x: i32,
    pub z: i32,
}

impl DHSectionPos {
    pub fn to_region_pos(self) -> RegionPos {
        RegionPos {
            x: self.x >> 3,
            z: self.z >> 3,
        }
    }
}

pub trait DHDataRequester {
    #[allow(unused)]
    fn get_section_poses(&self) -> Result<Vec<DHSectionPos>>;

    fn request_sections_in_region(
        &self,
        pos: &RegionPos,
    ) -> Result<HashMap<DHSectionPos, DHSectionData>>;
}

#[allow(unused)]
#[derive(Debug)]
pub struct DHSectionData {
    pub pos: DHSectionPos,
    pub min_y: i32,
    pub data: Vec<Vec<DHFullDataPoint>>,
    pub mapping: Vec<DHMappingEntry>,
    pub data_format_version: i8,
    pub compression_mode: CompressionMode,
}

#[allow(unused)]
#[derive(Debug)]
pub struct DHMappingEntry {
    pub biome: String,
    pub block: Option<String>,
    pub block_state: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy)]
pub struct DHFullDataPoint {
    pub id: i32,
    pub height: i32,
    pub min_y: i32,
}

pub const DH_SECTION_WIDTH: usize = 64;

impl DHSectionData {}

pub fn deserialize_data(
    data: Vec<u8>,
    compression_mode: &CompressionMode,
) -> Result<Vec<Vec<DHFullDataPoint>>> {
    let data = compression_mode.decompress(data)?;
    let mut data = Cursor::new(data);
    let mut data_list =
        Vec::<Vec<DHFullDataPoint>>::with_capacity(DH_SECTION_WIDTH * DH_SECTION_WIDTH);
    for xz in 0..DH_SECTION_WIDTH * DH_SECTION_WIDTH {
        let data_col_len = data.read_i16::<BigEndian>()?;
        ensure!(
            data_col_len >= 0,
            "Read DataSource Blob data at index [{xz}], column length [{data_col_len}] should be greater than zero."
        );
        let mut data_col = Vec::<DHFullDataPoint>::new();
        for _ in 0..data_col_len {
            let data = data.read_i64::<BigEndian>()?;
            data_col.push(DHFullDataPoint {
                id: (data & 2147483647i64) as i32,
                height: ((data >> 32i64) & 4095i64) as i32,
                min_y: ((data >> 44i64) & 4095i64) as i32,
            });
        }
        data_list.push(data_col);
    }
    Ok(data_list)
}

pub fn deserialize_mapping(
    data: Vec<u8>,
    compression_mode: &CompressionMode,
) -> Result<Vec<DHMappingEntry>> {
    let data = compression_mode.decompress(data)?;
    let mut data = Cursor::new(data);
    let state_len = data.read_i32::<BigEndian>()?;
    ensure!(state_len > 0, "There are no mapping.");
    let mut mapping = Vec::<DHMappingEntry>::new();
    for _ in 0..state_len {
        let utf_len = data.read_i16::<BigEndian>()?;
        let mut buf = vec![0u8; utf_len as usize];
        data.read_exact(&mut buf)?;
        let read = String::from_utf8(buf)?;
        if !read.contains("_DH-BSW_") {
            bail!("Failed to deserialize DHMappingEntry [{read}], unable to find separator.");
        }
        let (biome, block_state) = {
            let bb: Vec<&str> = read.split("_DH-BSW_").collect();
            (bb[0].to_string(), bb[1].to_string())
        };

        if block_state.contains("_STATE_") {
            let (block, states) = {
                let b: Vec<&str> = block_state.split("_STATE_").collect();
                (b[0].to_string(), b[1].to_string())
            };
            if block == "AIR" {
                mapping.push(DHMappingEntry {
                    biome,
                    block: None,
                    block_state: HashMap::new(),
                });
                continue;
            }
            if states.len() <= 0 {
                mapping.push(DHMappingEntry {
                    biome,
                    block: Some(block),
                    block_state: HashMap::new(),
                });
                continue;
            }
            let mut state_dict = HashMap::new();
            for state in states[1..states.len() - 1].split("}{") {
                let (key, value) = {
                    let kv: Vec<&str> = state.split(":").collect();
                    (kv[0].to_string(), kv[1].to_string())
                };
                state_dict.insert(key, value);
            }
            mapping.push(DHMappingEntry {
                biome,
                block: Some(block),
                block_state: state_dict,
            });
        } else {
            let block = block_state;
            if block == "AIR" {
                mapping.push(DHMappingEntry {
                    biome,
                    block: None,
                    block_state: HashMap::new(),
                });
                continue;
            } else {
                mapping.push(DHMappingEntry {
                    biome,
                    block: Some(block),
                    block_state: HashMap::new(),
                });
                continue;
            }
        }
    }
    Ok(mapping)
}
