// src/main.rs
mod cleanup;
mod io;
mod pipeline;

use anyhow::Result;
use colored::*;
use env_logger;
use indicatif::ProgressBar;
use log::{debug, error, info, warn};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    env_logger::init();
    info!("{}", "Starting French→English translator...".green());

    let input_path = "res/french.txt";
    let output_path = "res/english.txt";
    let cache_dir = ".cache";

    if !Path::new(cache_dir).exists() {
        fs::create_dir(cache_dir)?;
        info!(
            "{}",
            format!("Created cache directory at {}", cache_dir).yellow()
        );
    }

    let raw_text = io::read_file(input_path)?;
    let paragraphs = cleanup::clean_text(&raw_text);
    let total_chunks = (paragraphs.len() + 3) / 4;
    let bar = ProgressBar::new(total_chunks as u64);

    for (i, chunk) in paragraphs.chunks(4).enumerate() {
        let chunk_file = format!("{}/english.{:02}.txt", cache_dir, i);
        if Path::new(&chunk_file).exists() {
            debug!("Skipping existing chunk: {}", chunk_file);
            bar.inc(1);
            continue;
        }

        match pipeline::translate_paragraphs(chunk) {
            Ok(translations) => {
                io::write_file(&chunk_file, &translations)?;
                info!("{}", format!("Saved chunk to {}", chunk_file).cyan());
            }
            Err(e) => {
                error!("Failed to translate chunk {}: {}", i, e);
                break;
            }
        }
        bar.inc(1);
    }

    bar.finish_with_message("Translation complete. Merging chunks...");
    io::merge_chunks(cache_dir, output_path)?;
    info!(
        "{}",
        format!("Saved final translation to {}", output_path).green()
    );
    Ok(())
}
