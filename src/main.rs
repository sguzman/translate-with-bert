mod io;
mod pipeline;

use anyhow::Result;
use colored::*;
use env_logger::Env;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info};
use rayon::prelude::*;
use std::{fs, path::Path};

fn main() -> Result<()> {
    // init logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("{}", "🚀 Starting French→English translator...".green());

    let input = Path::new("res/french.txt");
    let cache = Path::new(".cache");
    let output = Path::new("res/english.txt");

    fs::create_dir_all(cache)?;
    info!("📂 Cache dir ready: {}", cache.display());

    // 1) Read & split
    let text = fs::read_to_string(input)?;
    let sentences = io::split_into_sentences(&text);
    let chunk_size = 4;
    let chunks: Vec<_> = sentences.chunks(chunk_size).map(|c| c.to_vec()).collect();
    let total = chunks.len();

    // 2) Progress bar
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} {pos:>4}/{len:4} [{bar:40.cyan/blue}] {elapsed_precise}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    // 3) Parallel translate & cache
    chunks.into_par_iter().enumerate().for_each(|(i, chunk)| {
        let path = cache.join(format!("english.{:02}.txt", i));
        if path.exists() {
            pb.inc(1);
            return;
        }
        info!("🔄 Translating chunk {:02}", i);
        match pipeline::translate_chunk(&chunk) {
            Ok(translated) => {
                if let Err(e) = io::write_chunk(&path, &translated) {
                    error!("Failed writing chunk {:02}: {}", i, e);
                } else {
                    info!("✅ Saved chunk {:02}", i);
                }
            }
            Err(e) => error!("❌ Chunk {:02} failed: {}", i, e),
        }
        pb.inc(1);
    });

    pb.finish_with_message("✨ All chunks processed");

    // 4) Merge
    io::merge_chunks(cache, output, total)?;
    info!("🎉 Final output at {}", output.display());

    Ok(())
}
