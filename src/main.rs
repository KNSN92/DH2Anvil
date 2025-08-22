mod data;
mod decompress;
mod sql;
mod worldgen;

use std::error::Error;

use sql::DHDBConn;

use crate::data::DHSectionPos;

fn main() -> Result<(), Box<dyn Error>> {
    let conn =
        DHDBConn::get_conn("./DistantHorizons.sqlite").expect("Failed to connect the database");
    let section_poses = conn.get_section_poses()?;
    println!("{:?}", conn.get_section_poses()?.len());
    let mut sections = conn.get_section(&DHSectionPos { x: 0, z: 0 })?;
    let section = sections.remove(0);
    println!("{:?}", section.mapping);

    Ok(())
}
