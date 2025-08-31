use std::{
    collections::HashMap,
    fs::metadata,
    path::Path,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use indicatif::{HumanBytes, MultiProgress, ProgressBar, ProgressStyle};

use clap::Parser;

use crate::worldgen::WorldGenStatus;

#[derive(Debug, Parser)]
#[command(version, about)]
/// Command-line arguments for the application.
///
/// # Fields
/// - `out`: Specifies the output directory for generated `.mca` files.
///   Defaults to `./region`.
/// - `threads`: Number of threads to use for world generation.
///   Set to `0` to automatically select the optimal number based on available CPU cores.
/// - `db_path`: Path to the input `.sqlite` file containing world data.
pub struct Args {
    #[arg(short, long, default_value_t = String::from("./region"), help="Specifies the output directory for generated `.mca` files.")]
    pub out: String,
    #[arg(
        short,
        long,
        default_value_t = 0,
        help = "Number of threads to use for world generation. Set to 0 for automatic selection based on available CPU cores."
    )]
    pub threads: u8,
    #[arg(
        short,
        long,
        default_value_t = 0,
        help = "Limits the generation range of region coordinates. If set to 0, all regions are generated. If set to 1 or higher, only regions where x and z are in the range -range to range-1 are generated."
    )]
    pub range: u32,
    #[arg(help = "Path to the input `.sqlite` file containing world data.")]
    pub db_path: String,
}

struct GeneratingRegionInfo {
    size: u64,
    generated: u64,
    thread_idx: usize,
    progressbar: ProgressBar,
}

pub fn start_progressbar(
    regions_count: u64,
    out_dir: impl AsRef<Path>,
    status_receiver: Receiver<WorldGenStatus>,
) -> Sender<()> {
    let out_dir = out_dir.as_ref().to_path_buf();
    let (stop_sender, stop_receiver) = mpsc::channel::<()>();
    thread::spawn(move || {
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
        loop {
            if let Ok(_) = stop_receiver.try_recv() {
                progresses.clear().unwrap();
                break;
            }
            if let Ok(status) = status_receiver.try_recv() {
                match status {
                    WorldGenStatus::StartRegion { pos, thread_idx } => {
                        let progressbar = ProgressBar::new(64);
                        progressbar.set_style(style.clone());
                        let progressbar = progresses.add(progressbar);
                        generating_regions.insert(
                            pos,
                            GeneratingRegionInfo {
                                size: 0,
                                generated: 0,
                                thread_idx,
                                progressbar,
                            },
                        );
                    }
                    WorldGenStatus::FinishDHSection { pos } => {
                        all_progress.inc(1);
                        let region_pos = pos.to_region_pos();
                        let region_file_path =
                            out_dir.join(format!("r.{}.{}.mca", region_pos.x, region_pos.z));
                        let file_size = metadata(region_file_path).unwrap().len();
                        let region_info = generating_regions.get_mut(&region_pos).unwrap();
                        region_info.size = file_size;
                        region_info.generated += 1;
                        region_info.progressbar.inc(1);
                        region_info.progressbar.set_message(format!(
                            "[x:{} z:{}] [region x:{:>3} z:{:>3}] [thread:{}] {}",
                            pos.x - (region_pos.x << 3),
                            pos.z - (region_pos.z << 3),
                            region_pos.x,
                            region_pos.z,
                            region_info.thread_idx,
                            HumanBytes(file_size).to_string()
                        ));
                    }
                    WorldGenStatus::FinishRegion { pos } => {
                        if let Some(region_info) = generating_regions.remove(&pos) {
                            total_generated_size += region_info.size;
                            progresses.remove(&region_info.progressbar);
                        }
                    }
                }
                let total_size =
                    total_generated_size + generating_regions.values().map(|v| v.size).sum::<u64>();
                all_progress.set_message(HumanBytes(total_size).to_string());
            }

            thread::sleep(Duration::from_millis(5));
        }
    });
    stop_sender
}
