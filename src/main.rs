mod io;
mod pipeline;

use anyhow::Result;
use colored::*;
use env_logger::Env;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info};
use std::{fs, path::Path};

fn main() -> Result<()> {
    // init logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("{}", "🚀 French→English (GPU batch)…".green());

    let input = Path::new("res/french.txt");
    let cache = Path::new(".cache");
    let output = Path::new("res/english.txt");

    fs::create_dir_all(cache)?;
    info!("📂 Cache dir ready: {}", cache.display());

    // 1) Read & split
    let text = fs::read_to_string(input)?;
    let sentences = io::split_into_sentences(&text);
    let chunk_size = 4;
    let chunks: Vec<Vec<String>> = sentences.chunks(chunk_size).map(|c| c.to_vec()).collect();
    let total = chunks.len();

    // 2) Progress bar
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} {pos:>4}/{len:4} [{bar:40.cyan/blue}] {elapsed_precise}",
        )?
        .progress_chars("##-"),
    );

    // 3) Identify missing chunks
    let mut missing_idx = Vec::new();
    for i in 0..total {
        let path = cache.join(format!("english.{:02}.txt", i));
        if path.exists() {
            pb.inc(1);
        } else {
            missing_idx.push(i);
        }
    }

    // 4) Batch-translate all missing chunks at once
    if !missing_idx.is_empty() {
        let batch_inputs: Vec<String> = missing_idx.iter().map(|&i| chunks[i].join(" ")).collect();

        info!("🔄 Batch-translating {} chunk(s)…", missing_idx.len());
        let translations = pipeline::translate_chunks(&batch_inputs)?;

        // 5) Write each translated chunk
        for (j, &i) in missing_idx.iter().enumerate() {
            let path = cache.join(format!("english.{:02}.txt", i));
            if let Err(e) = io::write_chunk(&path, &[translations[j].clone()]) {
                error!("Failed writing chunk {:02}: {}", i, e);
            } else {
                info!("✅ Saved chunk {:02}", i);
            }
            pb.inc(1);
        }
    }

    pb.finish_with_message("✨ All chunks processed");

    // 6) Merge into final file
    io::merge_chunks(cache, output, total)?;
    info!("🎉 Final output at {}", output.display());

    Ok(())
}
