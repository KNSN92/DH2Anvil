use std::path::Path;

use anyhow::{Result, bail};
use rusqlite::Connection;

use crate::{
    data::{DHSectionData, DHSectionPos, deserialize_data, deserialize_mapping},
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

    pub fn get_section(&self, pos: &DHSectionPos) -> Result<Vec<DHSectionData>> {
        let mut stmt = self.0.prepare_cached(
            "SELECT PosX, PosZ, MinY, Data, Mapping, DataFormatVersion, CompressionMode FROM FullData WHERE DetailLevel = 0 and PosX = $pos_x and PosZ = $pos_z;"
        )?;
        let raw_sections_iter = stmt.query_map([pos.x, pos.z], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            ))
        })?;
        let mut sections = Vec::new();
        for raw_section in raw_sections_iter {
            let (pos_x, pos_z, min_y, data, mapping, data_format_version, compression_mode_num) =
                raw_section?;
            let compression_mode = CompressionMode::from_num(compression_mode_num);
            let compression_mode = if let Some(compression_mode) = compression_mode {
                compression_mode
            } else {
                bail!("Invalid compression mode number {compression_mode_num}")
            };
            sections.push(DHSectionData {
                pos: DHSectionPos { x: pos_x, z: pos_z },
                min_y,
                data: deserialize_data(data, &compression_mode)?,
                mapping: deserialize_mapping(mapping, &compression_mode)?,
                data_format_version,
                compression_mode,
            });
        }
        Ok(sections)
    }
}
