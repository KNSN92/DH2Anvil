mod chunk;
mod cli;
mod data;
mod decompress;
mod sql;
mod worldgen;
mod block_entity;

use std::{
    collections::HashSet,
    fs::create_dir_all,
    path::Path,
    sync::{
        Mutex,
        mpsc::{self},
    },
};

use anyhow::{Result, ensure};
use clap::Parser;
use console::style;
use rayon::{
    ThreadPoolBuilder,
    iter::{IntoParallelIterator, ParallelIterator},
};
use sql::DHDBConn;

use crate::{
    cli::{Args, Commands, start_progressbar},
    data::RegionPos,
    worldgen::{generate_world, get_region_filename},
};

fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Commands::Convert { db_path, out, threads, range, overwrite, no_blockentity } => {
            run_convert(db_path, out, threads, range, overwrite, no_blockentity)?;
        }
        Commands::Info { db_path } => {
            run_info(db_path)?;
        }
    }
    
    Ok(())
}

fn run_convert(
    db_path: String,
    out: String,
    threads: u8,
    range: u32,
    overwrite: bool,
    no_blockentity: bool
) -> Result<()> {
    let db_path = Path::new(&db_path);
    ensure!(
        db_path.exists(),
        format!("DH Lod data file '{}' does not exists", db_path.display())
    );
    if threads > 0 {
        ThreadPoolBuilder::new()
            .num_threads(threads as usize)
            .build_global()
            .unwrap();
    }
    let conn = DHDBConn::get_conn(db_path)?;
    let mut region_poses: Vec<_> = conn
        .get_section_poses()?
        .into_par_iter()
        .map(RegionPos::from)
        .filter(|pos| {
            let limit = range as i64;
            range == 0
                || (-limit..limit).contains(&(pos.x as i64))
                    && (-limit..limit).contains(&(pos.z as i64))
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    
    let out_dir = Path::new(&out);
    
    let conn = Mutex::new(conn);
    create_dir_all(out_dir)?;
    let (status_sender, status_receiver) = mpsc::channel();
    let skipped_regions = if !overwrite {
        let full_region_count = region_poses.len();
        region_poses = region_poses
            .into_par_iter()
            .filter(|region_pos| {
                let region_file = out_dir.join(get_region_filename(region_pos));
                !region_file.exists()
            })
            .collect();
        full_region_count - region_poses.len()
    } else {
        0
    };
    let stop_progressbar = start_progressbar(
        region_poses.len() as u64,
        skipped_regions as u64,
        out_dir,
        status_receiver,
    );
    generate_world(region_poses, conn, out_dir, overwrite, no_blockentity, status_sender)?;
    stop_progressbar();
    Ok(())
}

fn run_info(db_path: String) -> Result<()> {
    let db_path = Path::new(&db_path);
    ensure!(
        db_path.exists(),
        format!("DH Lod data file '{}' does not exists", db_path.display())
    );
    
    println!("{}", style("=== Database Information ===").bold().cyan());
    println!("Database file: {}", db_path.display());
    
    let conn = DHDBConn::get_conn(db_path)?;
    let stats = conn.get_database_stats()?;
    
    let total_regions = conn
        .get_section_poses()?
        .into_par_iter()
        .map(RegionPos::from)
        .collect::<HashSet<_>>()
        .len();
    
    println!("\n{}", style("Statistics:").bold());
    println!("  Total sections:     {}", stats.total_sections);
    println!("  Total regions:      {}", total_regions);
    
    println!("\n{}", style("Coordinate Range:").bold());
    println!("  Section X: {} to {}", stats.min_section_x, stats.max_section_x);
    println!("  Section Z: {} to {}", stats.min_section_z, stats.max_section_z);
    
    let region_min_x = stats.min_section_x >> 3;
    let region_max_x = stats.max_section_x >> 3;
    let region_min_z = stats.min_section_z >> 3;
    let region_max_z = stats.max_section_z >> 3;
    println!("  Region X:  {} to {}", region_min_x, region_max_x);
    println!("  Region Z:  {} to {}", region_min_z, region_max_z);
    
    Ok(())
}
