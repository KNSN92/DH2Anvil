use std::{
    collections::HashMap,
    fs::metadata,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, SystemTime},
};

use indicatif::{HumanBytes, MultiProgress, ProgressBar, ProgressStyle};

use clap::{Parser, Subcommand};

use crate::{data::RegionPos, worldgen::WorldGenStatus};

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Convert DH LOD data to Anvil format")]
    Convert {
        #[arg(short, long, default_value_t = String::from("./region"), help="Specifies the output directory for generated `.mca` files.")]
        out: String,
        #[arg(
            short,
            long,
            default_value_t = 0,
            help = "Number of threads to use for world generation. Set to 0 for automatic selection based on available CPU cores."
        )]
        threads: u8,
        #[arg(
            short,
            long,
            default_value_t = 0,
            help = "Limits the generation range of region coordinates. If set to 0, all regions are generated. If set to 1 or higher, only regions where x and z are in the range -range to range-1 are generated."
        )]
        range: u32,
        #[arg(
            long,
            default_value_t = false,
            help = "Whether to overwrite an existing file."
        )]
        overwrite: bool,
        #[arg(help = "Path to the input `.sqlite` file containing dh lod data.")]
        db_path: String,
    },
    #[command(about = "Display information about the database")]
    Info {
        #[arg(help = "Path to the input `.sqlite` file containing dh lod data.")]
        db_path: String,
    },
}

struct GeneratingRegionInfo {
    size: u64,
    thread_idx: Option<usize>,
    file_path: PathBuf,
    progressbar: ProgressBar,
}

pub fn start_progressbar(
    regions_count: u64,
    skipped_regions: u64,
    out_dir: impl AsRef<Path>,
    status_receiver: Receiver<WorldGenStatus>,
) -> impl FnOnce() {
    let out_dir = out_dir.as_ref().to_path_buf();
    let (stop_sender, stop_receiver) = mpsc::channel::<()>();
    let handle = thread::spawn(move || {
        let style = ProgressStyle::default_bar()
            .template(
                "[{elapsed_precise}] {spinner} [{eta}] [{bar:40.green/blue}] {pos}/{len} {msg}",
            )
            .unwrap()
            .progress_chars("=>..");
        let progresses = MultiProgress::new();
        let all_progress = progresses.add(ProgressBar::new(regions_count * 64));
        all_progress.set_style(style);

        let style = ProgressStyle::default_bar()
            .template("{spinner} [{pos:>2}/{len}] {msg}")
            .unwrap()
            .progress_chars("..  ");
        let mut generating_regions = HashMap::new();
        let mut total_generated_size = 0u64;
        let mut skipped_regions = skipped_regions;
        loop {
            let now = SystemTime::now();
            if let Ok(status) = status_receiver.try_recv() {
                match status {
                    WorldGenStatus::StartRegion {
                        pos,
                        thread_idx,
                        file_path,
                    } => {
                        let progressbar = ProgressBar::new(64);
                        progressbar.set_style(style.clone());
                        let progressbar = progresses.add(progressbar);
                        generating_regions.insert(
                            pos,
                            GeneratingRegionInfo {
                                size: 0,
                                thread_idx,
                                file_path,
                                progressbar,
                            },
                        );
                    }
                    WorldGenStatus::FinishDHSection { pos } => {
                        all_progress.inc(1);
                        let region_pos = RegionPos::from(pos);
                        let region_info = generating_regions.get_mut(&region_pos).unwrap();
                        let file_size = metadata(&region_info.file_path)
                            .map(|meta| meta.len())
                            .unwrap_or(0);
                        region_info.size = file_size;
                        region_info.progressbar.inc(1);
                        region_info.progressbar.set_message(format!(
                            "[x:{} z:{}] [region x:{:>3} z:{:>3}] [thread:{}] {}",
                            pos.x - (region_pos.x << 3),
                            pos.z - (region_pos.z << 3),
                            region_pos.x,
                            region_pos.z,
                            region_info
                                .thread_idx
                                .map_or("?".to_string(), |idx| idx.to_string()),
                            HumanBytes(file_size)
                        ));
                    }
                    WorldGenStatus::FinishRegion { pos } => {
                        if let Some(region_info) = generating_regions.remove(&pos) {
                            let region_file_path =
                                out_dir.join(format!("r.{}.{}.mca", pos.x, pos.z));
                            let file_size = metadata(region_file_path).unwrap().len();
                            total_generated_size += file_size;
                            region_info.progressbar.finish_and_clear();
                            progresses.remove(&region_info.progressbar);
                        }
                    }
                    WorldGenStatus::SkipRegion => {
                        all_progress.inc(64);
                        skipped_regions += 1;
                    }
                }
                let total_size =
                    total_generated_size + generating_regions.values().map(|v| v.size).sum::<u64>();
                all_progress.set_message(HumanBytes(total_size).to_string());
            }
            if stop_receiver.try_recv().is_ok() {
                if skipped_regions > 0 {
                    println!(
                        "{skipped_regions} region files were skipped because a file with the same name already exists!\nTips: If you need to overwrite an existing mca file, add the \"--overwrite\" option to the command!\nDone ✨"
                    );
                } else {
                    println!("Done ✨");
                }
                return;
            }
            let elapsed = now.elapsed().unwrap();
            thread::sleep(
                Duration::from_millis(10)
                    .checked_sub(elapsed)
                    .unwrap_or_else(|| Duration::from_millis(10)),
            );
        }
    });
    move || {
        stop_sender.send(()).unwrap();
        handle.join().unwrap();
    }
}
