mod chunk;
mod data;
mod decompress;
mod sql;
mod worldgen;

use std::error::Error;

use indicatif::{ProgressBar, ProgressStyle};
use sql::DHDBConn;
use worldgen::DHData2WorldGenerator;

fn main() -> Result<(), Box<dyn Error>> {
    let conn =
        DHDBConn::get_conn("./DistantHorizons.sqlite").expect("Failed to connect the database");
    let section_poses = conn.get_section_poses()?;
    // ここからの行消したらワールドの全変換開始する
    let section_poses: Vec<_> = section_poses
        .into_iter()
        .filter(|section_pos| {
            -4 <= section_pos.x && 4 > section_pos.x && -4 <= section_pos.z && 4 > section_pos.z
        })
        .collect();
    // ここまで
    let style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{eta}] [{bar:40.green/blue}] {pos}/{len} {msg}")?
        .progress_chars("==>..");
    let progress = ProgressBar::new(section_poses.len() as u64);
    progress.set_style(style);
    let mut generator = DHData2WorldGenerator::new("./out", section_poses, move |pos| {
        let section_data = conn.get_section(pos).unwrap();
        section_data.into_iter().next().unwrap()
    })?;
    generator.generate(|_, size| {
        progress.inc(1);
        progress.set_message(format!("{:.1}MB", size as f32 / (1024 * 1024) as f32));
    })?;
    Ok(())
}
