use std::{
    collections::{BTreeMap, HashMap},
    u32,
};

use anyhow::{Result, bail};
use fastnbt::{LongArray, Value};
use serde::{Deserialize, Serialize};

pub const AIR: &str = "minecraft:air";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    #[serde(rename = "xPos")]
    pub x: i32,
    #[serde(rename = "zPos")]
    pub z: i32,
    #[serde(rename = "Status")]
    pub status: String,
    pub sections: Vec<Section>,

    #[serde(flatten)]
    other: HashMap<String, Value>,
}

impl Chunk {
    pub fn set_chunk_pos(&mut self, x: i32, z: i32) {
        self.x = x;
        self.z = z;
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }

    pub fn set_block(&mut self, x: u32, y: i32, z: u32, block: BlockState) -> Result<()> {
        if 16 <= x || 16 <= z || y < -64 || 320 <= y {
            bail!("x or y or z is out of bounds x:{x} y:{y} z:{z}");
        }
        let section = &mut self.sections[((y as f32 / 16.).floor() + 4.) as usize];
        let id = if let Some(id) = section.block_states.rev_palette.get(&block) {
            *id
        } else {
            let id = section.block_states.palette.len() as u16;
            section.block_states.palette.push(block.clone());
            section.block_states.rev_palette.insert(block, id);
            id
        };
        section.block_states.data[(((y & 15) as u32) << 8 | z << 4 | x) as usize] = id;
        Ok(())
    }

    pub fn set_biome(&mut self, biome: String) {
        self.sections.iter_mut().for_each(|section| {
            section.biomes.data = None;
            section.biomes.palette = vec![biome.clone()];
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    #[serde(rename = "Y")]
    pub y: i8,
    pub block_states: BlockStates,
    pub biomes: Biomes,
}

#[derive(Debug, Serialize, Deserialize)]
struct _BlockStates {
    palette: Vec<BlockState>,
    data: Option<LongArray>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "_BlockStates", into = "_BlockStates")]
pub struct BlockStates {
    palette: Vec<BlockState>,
    #[serde(skip)]
    rev_palette: HashMap<BlockState, u16>,
    data: Vec<u16>,
}

impl Into<_BlockStates> for BlockStates {
    fn into(self) -> _BlockStates {
        let data = self.data;
        let mut new_data = vec![0; data.len()];
        let mut used_pallette_items = HashMap::<u16, usize>::new();
        let mut palette = Vec::new();
        for (i, data_item) in data.iter().enumerate() {
            let idx = if let Some(idx) = used_pallette_items.get(data_item) {
                *idx
            } else {
                let state = self.palette[*data_item as usize].clone();
                palette.push(state);
                let idx = used_pallette_items.len();
                used_pallette_items.insert(*data_item, idx);
                idx
            };
            new_data[i] = idx as i64;
        }
        let data = new_data;
        let palette = if palette.len() == 0 {
            vec![BlockState {
                name: AIR.to_string(),
                properties: None,
            }]
        } else {
            palette
        };
        let data = if palette.len() <= 1 {
            None
        } else {
            let bits = (usize::BITS - (palette.len() - 1).leading_zeros()).max(4);
            let blocks_par_i64 = i64::BITS / bits;
            let mut encoded_data = Vec::new();
            for (i, data_item) in data.iter().enumerate() {
                let current_ptr = i % blocks_par_i64 as usize * bits as usize;
                let encoded_data_item = if current_ptr == 0 {
                    encoded_data.push(0);
                    0i64
                } else {
                    *encoded_data.last().unwrap()
                };
                *encoded_data.last_mut().unwrap() = encoded_data_item | data_item << current_ptr;
            }
            Some(LongArray::new(encoded_data))
        };
        _BlockStates { palette, data }
    }
}

impl From<_BlockStates> for BlockStates {
    fn from(value: _BlockStates) -> Self {
        let palette = if value.palette.len() == 0 {
            vec![BlockState {
                name: AIR.to_string(),
                properties: None,
            }]
        } else {
            value.palette
        };
        let mut rev_palette = HashMap::new();
        for (i, state) in palette.iter().enumerate() {
            rev_palette.insert(state.clone(), i as u16);
        }
        let data = value.data;
        let data = if data == None {
            vec![0u16; 16 * 16 * 16]
        } else {
            let data = data.unwrap();
            let bits = (usize::BITS - (palette.len() - 1).leading_zeros()).max(4);
            let blocks_par_i64 = i64::BITS / bits;
            let mut decoded_data = Vec::new();
            for mut data_item in data.into_inner() {
                for _ in 0..blocks_par_i64 {
                    let id = (data_item & ((1 << bits) - 1)) as u16;
                    data_item = data_item >> bits;
                    decoded_data.push(id);
                }
            }
            decoded_data
        };
        BlockStates {
            palette,
            rev_palette,
            data,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct BlockState {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Properties")]
    pub properties: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Biomes {
    palette: Vec<String>,
    data: Option<Vec<i64>>,
}
