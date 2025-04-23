mod cleanup;
mod io;
mod pipeline;

use anyhow::Result;
use colored::*;
use env_logger;
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    env_logger::init();
    info!("{}", "🚀 Starting French→English translator...".green());

    let input_path = "res/french.txt";
    let output_path = "res/english.txt";
    let cache_dir = ".cache";

    if !Path::new(cache_dir).exists() {
        fs::create_dir(cache_dir)?;
        info!(
            "{}",
            format!("📂 Created cache directory: {}", cache_dir).blue()
        );
    }

    let raw_text = io::read_file(input_path)?;
    let paragraphs = cleanup::clean_text(&raw_text);
    let total_chunks = (paragraphs.len() + 3) / 4;

    let pb = ProgressBar::new(total_chunks as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} Chunks",
            )
            .unwrap()
            .progress_chars("##-"),
    );

    for (i, chunk) in paragraphs.chunks(4).enumerate() {
        let cache_file = format!("{}/english.{:02}.txt", cache_dir, i);
        if Path::new(&cache_file).exists() {
            pb.inc(1);
            continue;
        }

        info!("{}", format!("🔄 Translating chunk {:02}...", i).yellow());
        let translated = pipeline::translate_paragraphs(chunk)?;
        io::write_file(&cache_file, &translated)?;
        info!("{}", format!("✅ Saved to {}", cache_file).green());
        pb.inc(1);
    }

    pb.finish_with_message("🏁 All chunks processed!");

    io::merge_chunks(cache_dir, output_path)?;
    info!(
        "{}",
        format!("📘 Final output saved to {}", output_path).cyan()
    );

    Ok(())
}
