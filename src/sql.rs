use std::{collections::HashMap, path::Path, sync::Mutex};

use anyhow::{Result, bail};
use rusqlite::Connection;

use crate::{
    data::{
        DHDataRequester, DHSectionData, DHSectionPos, RegionPos, deserialize_data,
        deserialize_mapping,
    },
    decompress::CompressionMode,
};

pub struct DHDBConn(pub Connection);

impl DHDBConn {
    pub fn get_conn(file: impl AsRef<Path>) -> Result<DHDBConn> {
        Ok(DHDBConn(Connection::open(file)?))
    }

    pub fn get_section_poses(&self) -> Result<Vec<DHSectionPos>> {
        let mut stmt = self
            .0
            .prepare_cached("SELECT PosX, PosZ FROM FullData WHERE DetailLevel = 0")?;
        let poses_iter = stmt.query_map([], |row| {
            Ok(DHSectionPos {
                x: row.get(0)?,
                z: row.get(1)?,
            })
        })?;
        let mut poses = Vec::new();
        for pos in poses_iter {
            poses.push(pos?);
        }
        Ok(poses)
    }

    pub fn get_sections_in_region(
        &self,
        region_pos: &RegionPos,
    ) -> Result<HashMap<DHSectionPos, DHSectionData>> {
        let mut stmt = self.0.prepare_cached(
            "SELECT PosX, PosZ, MinY, Data, Mapping, DataFormatVersion, CompressionMode FROM FullData WHERE DetailLevel = 0 and $pos_x_min <= PosX and PosX < $pos_x_max and $pos_z_min <= PosZ and PosZ < $pos_z_max;"
        )?;
        let (section_min_x, section_min_z) = (region_pos.x << 3, region_pos.z << 3);
        let (section_max_x, section_max_z) = (section_min_x + 8, section_min_z + 8);
        let raw_sections_iter = stmt.query_map(
            [section_min_x, section_max_x, section_min_z, section_max_z],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )?;
        let mut sections = HashMap::new();
        for raw_section in raw_sections_iter {
            let (pos_x, pos_z, min_y, data, mapping, data_format_version, compression_mode_num) =
                raw_section?;
            let compression_mode = CompressionMode::from_num(compression_mode_num);
            let compression_mode = if let Some(compression_mode) = compression_mode {
                compression_mode
            } else {
                bail!("Invalid compression mode number {compression_mode_num}")
            };
            sections.insert(
                DHSectionPos { x: pos_x, z: pos_z },
                DHSectionData {
                    pos: DHSectionPos { x: pos_x, z: pos_z },
                    min_y,
                    data: deserialize_data(data, &compression_mode)?,
                    mapping: deserialize_mapping(mapping, &compression_mode)?,
                    data_format_version,
                    compression_mode,
                },
            );
        }
        Ok(sections)
    }
}

impl DHDataRequester for Mutex<DHDBConn> {
    fn get_section_poses(&self) -> Result<Vec<DHSectionPos>> {
        self.lock()
            .expect("Failed to lock DHDBConn it is poisoned")
            .get_section_poses()
    }

    fn request_sections_in_region(
        &self,
        pos: &RegionPos,
    ) -> Result<HashMap<DHSectionPos, DHSectionData>> {
        self.lock()
            .expect("Failed to lock DHDBConn it is poisoned")
            .get_sections_in_region(pos)
    }
}
