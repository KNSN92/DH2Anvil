mod chunk;
mod cli;
mod data;
mod decompress;
mod sql;
mod worldgen;

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
use rayon::{
    ThreadPoolBuilder,
    iter::{IntoParallelIterator, ParallelIterator},
};
use sql::DHDBConn;

use crate::{
    cli::{Args, start_progressbar},
    data::RegionPos,
    worldgen::{generate_world, get_region_filename},
};

fn main() -> Result<()> {
    let args = Args::parse();
    let db_path = Path::new(&args.db_path);
    ensure!(
        db_path.exists(),
        format!("DH Lod data file '{}' does not exists", args.db_path)
    );
    if args.threads > 0 {
        ThreadPoolBuilder::new()
            .num_threads(args.threads as usize)
            .build_global()
            .unwrap();
    }
    let conn = DHDBConn::get_conn(db_path)?;
    let mut region_poses: Vec<_> = conn
        .get_section_poses()?
        .into_par_iter()
        .map(RegionPos::from)
        .filter(|pos| {
            let limit = args.range as i64;
            args.range == 0
                || (-limit..limit).contains(&(pos.x as i64))
                    && (-limit..limit).contains(&(pos.z as i64))
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let conn = Mutex::new(conn);
    let out_dir = Path::new(&args.out);
    create_dir_all(out_dir)?;
    let (status_sender, status_receiver) = mpsc::channel();
    let skipped_regions = if !args.overwrite {
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
    generate_world(region_poses, conn, out_dir, args.overwrite, status_sender)?;
    stop_progressbar();
    Ok(())
}
