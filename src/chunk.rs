use std::collections::{BTreeMap, HashMap};

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

    pub fn set_block_biome(
        &mut self,
        x: u32,
        y: i32,
        z: u32,
        block: BlockState,
        biome: String,
    ) -> Result<()> {
        if 16 <= x || 16 <= z || !(-64..320).contains(&y) {
            bail!("x or y or z is out of bounds x:{x} y:{y} z:{z}");
        }
        let section = &mut self.sections[((y as f32 / 16.).floor() + 4.) as usize];

        let y = (y & 0xf) as u32;

        let id = if let Some(id) = section.block_states.rev_palette.get(&block) {
            *id
        } else {
            let id = section.block_states.palette.len() as u16;
            section.block_states.palette.push(block.clone());
            section.block_states.rev_palette.insert(block, id);
            id
        };
        section.block_states.data[(y << 8 | z << 4 | x) as usize] = id;

        let id = if let Some(id) = section.biomes.rev_palette.get(&biome) {
            *id
        } else {
            let id = section.biomes.palette.len() as u16;
            section.biomes.palette.push(biome.clone());
            section.biomes.rev_palette.insert(biome, id);
            id
        };
        section.biomes.data[(y << 8 | z << 4 | x) as usize] = id;

        Ok(())
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

impl From<BlockStates> for _BlockStates {
    fn from(value: BlockStates) -> Self {
        let data = value.data;
        let mut new_data = vec![0; data.len()];
        let mut used_pallette_items = HashMap::<u16, usize>::new();
        let mut palette = Vec::new();
        for (i, data_item) in data.iter().enumerate() {
            let idx = if let Some(idx) = used_pallette_items.get(data_item) {
                *idx
            } else {
                let state = value.palette[*data_item as usize].clone();
                palette.push(state);
                let idx = used_pallette_items.len();
                used_pallette_items.insert(*data_item, idx);
                idx
            };
            new_data[i] = idx as u16;
        }
        let data = new_data;
        let palette = if palette.is_empty() {
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
            Some(LongArray::new(pack_data(data, &palette.len(), 4)))
        };
        _BlockStates { palette, data }
    }
}

impl From<_BlockStates> for BlockStates {
    fn from(value: _BlockStates) -> Self {
        let palette = if value.palette.is_empty() {
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
        let data = if let Some(data) = data {
            unpack_data(data.into_inner(), &palette.len(), 4)
        } else {
            vec![0u16; 16 * 16 * 16]
        };
        BlockStates {
            palette,
            rev_palette,
            data,
        }
    }
}

fn pack_data(data: Vec<u16>, palette_len: &usize, min_bits: u32) -> Vec<i64> {
    let bits = (usize::BITS - (palette_len - 1).leading_zeros()).max(min_bits);
    let data_par_i64 = i64::BITS / bits;
    let mut packed_data = Vec::new();
    for (i, data_item) in data.iter().enumerate() {
        let data_item = *data_item as i64;
        let current_ptr = i % data_par_i64 as usize * bits as usize;
        let encoded_data_item = if current_ptr == 0 {
            packed_data.push(0);
            0i64
        } else {
            *packed_data.last().unwrap()
        };
        *packed_data.last_mut().unwrap() = encoded_data_item | data_item << current_ptr;
    }
    packed_data
}

fn unpack_data(data: Vec<i64>, palette_len: &usize, min_bits: u32) -> Vec<u16> {
    let bits = (usize::BITS - (palette_len - 1).leading_zeros()).max(min_bits);
    let blocks_par_i64 = i64::BITS / bits;
    let mut unpacked_data = Vec::new();
    for mut data_item in data {
        for _ in 0..blocks_par_i64 {
            let id = (data_item & ((1 << bits) - 1)) as u16;
            data_item >>= bits;
            unpacked_data.push(id);
        }
    }
    unpacked_data
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct BlockState {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Properties")]
    pub properties: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _Biomes {
    palette: Vec<String>,
    data: Option<LongArray>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "_Biomes", into = "_Biomes")]
pub struct Biomes {
    palette: Vec<String>,
    #[serde(skip)]
    rev_palette: HashMap<String, u16>,
    data: Vec<u16>,
}

impl From<Biomes> for _Biomes {
    fn from(value: Biomes) -> Self {
        let data = value.data;
        let mut new_data = vec![0; data.len()];
        let mut used_pallette_items = HashMap::<u16, usize>::new();
        let mut palette = Vec::new();
        for (i, data_item) in data.iter().enumerate() {
            let idx = if let Some(idx) = used_pallette_items.get(data_item) {
                *idx
            } else {
                let state = value.palette[*data_item as usize].clone();
                palette.push(state);
                let idx = used_pallette_items.len();
                used_pallette_items.insert(*data_item, idx);
                idx
            };
            new_data[i] = idx as u16;
        }
        let data = new_data;
        let palette = if palette.is_empty() {
            vec![String::from("minecraft:plains")]
        } else {
            palette
        };
        let data = if palette.len() <= 1 {
            None
        } else {
            let mut new_data = Vec::new();
            for i in 0..(4 * 4 * 4) {
                let mut biomes = Vec::new();
                let x = (i & 3) << 2;
                let z = ((i >> 2) & 3) << 2;
                let y = ((i >> 4) & 3) << 2;
                for p in 0..64 {
                    let y = y | (p >> 4) & 3;
                    let z = z | (p >> 2) & 3;
                    let x = x | p & 3;
                    biomes.push(data[y << 4 | z << 2 | x]);
                }
                if i % 2 == 0 {
                    new_data.push(*biomes.first().unwrap());
                } else {
                    new_data.push(*biomes.last().unwrap());
                }
            }
            Some(LongArray::new(pack_data(new_data, &palette.len(), 0)))
        };
        _Biomes { palette, data }
    }
}

impl From<_Biomes> for Biomes {
    fn from(value: _Biomes) -> Self {
        let palette = if value.palette.is_empty() {
            vec![String::from("minecraft:plains")]
        } else {
            value.palette
        };
        let mut rev_palette = HashMap::new();
        for (i, state) in palette.iter().enumerate() {
            rev_palette.insert(state.clone(), i as u16);
        }
        let data = value.data;
        let data = if let Some(data) = data {
            let data = unpack_data(data.into_inner(), &palette.len(), 0);
            let mut new_data = vec![0; data.len() * (4 * 4 * 4)];
            for (i, data_item) in data.into_iter().enumerate() {
                let x = (i & 3) << 2;
                let z = ((i >> 2) & 3) << 2;
                let y = ((i >> 4) & 3) << 2;
                for p in 0..64 {
                    let y = y | (p >> 4) & 3;
                    let z = z | (p >> 2) & 3;
                    let x = x | p & 3;
                    new_data[y << 4 | z << 2 | x] = data_item;
                }
            }
            new_data
        } else {
            vec![0u16; 16 * 16 * 16]
        };
        Biomes {
            palette,
            rev_palette,
            data,
        }
    }
}
