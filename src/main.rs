// src/main.rs

mod cleanup;
mod io;
mod pipeline; // ensure cleanup is in scope

use anyhow::Result;

use env_logger::Env;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info, warn};
use std::{fs, path::Path};
use tch::Device;

// Import colored extensions
use colored::Colorize;

fn main() -> Result<()> {
    // 0) Init logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // 1) Verify GPU availability
    let device = Device::cuda_if_available();
    match device {
        Device::Cuda(_) => info!("🖥️  Using device: {:?}", device),
        Device::Cpu => {
            error!(
                "❌ No CUDA GPU available (detected {:?}). Aborting.",
                device
            );
            std::process::exit(1);
        }
        _ => info!("🖥️  Using device: {:?}", device),
    }

    info!(
        "{}",
        "🚀 Starting French→English translator (GPU batch)…".green()
    );

    // 2) Paths
    let input = Path::new("res/german.txt");
    let cache = Path::new(".cache");
    let output = Path::new("res/english.txt");

    fs::create_dir_all(cache)?;
    info!("📂 Cache dir ready: {}", cache.display());

    // 3) Read & split into paragraphs
    let text = fs::read_to_string(input)?;
    let mut paragraphs = io::split_into_paragraphs(&text);

    // 3a) For long paragraphs, break into sliding windows
    let max_sents = 32;
    let overlap = 8;
    let mut chunks = Vec::new();
    for para in paragraphs.drain(..) {
        let sents = io::split_into_sentences(&para);
        if sents.len() <= max_sents {
            chunks.push(para.clone());
        } else {
            let mut start = 0;
            while start < sents.len() {
                let end = usize::min(start + max_sents, sents.len());
                chunks.push(sents[start..end].join(" "));
                if end == sents.len() {
                    break;
                }
                start += max_sents - overlap;
            }
        }
    }

    let total = chunks.len();

    // 4) Progress bar
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} {pos:>4}/{len:4} [{bar:40.cyan/blue}] {elapsed_precise}",
        )?
        .progress_chars("##-"),
    );

    // 5) Find missing chunks
    let mut missing_idx = Vec::new();
    for i in 0..total {
        let p = cache.join(format!("english.{:02}.txt", i));
        if p.exists() {
            pb.inc(1);
        } else {
            missing_idx.push(i);
        }
    }

    // 6) Configurable batch size (via BATCH_SIZE env, default 4)
    let batch_size = std::env::var("BATCH_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(4);
    info!(
        "🔄 Translating {} missing chunk(s) in batches of {}…",
        missing_idx.len(),
        batch_size
    );

    // 7) Process in batches
    for batch in missing_idx.chunks(batch_size) {
        let batched_inputs: Vec<String> = batch.iter().map(|&i| chunks[i].clone()).collect();
        let now = std::time::Instant::now();
        let translations = pipeline::translate_chunks(&batched_inputs)?;
        let elapsed = now.elapsed();

        let timed = format!(
            "🔡 Translated {} chunks in {} ms ({} seconds)",
            batch.len(),
            elapsed.as_millis(), // Corrected from as_millis() to as_millis()
            elapsed.as_secs()
        );
        warn!("{}", timed.yellow());

        for (j, &i) in batch.iter().enumerate() {
            let path = cache.join(format!("english.{:02}.txt", i));
            let translated_text = &translations[j];
            let lines: Vec<String> = translated_text.lines().map(str::to_string).collect();

            if let Err(e) = io::write_chunk(&path, &lines) {
                error!("❌ Failed writing chunk {:02}: {}", i, e);
            } else {
                info!("✅ Saved chunk {:02}", i);
            }
            pb.inc(1);
        }
    }

    pb.finish_with_message("✨ All chunks processed");

    // 8) Merge into final file
    io::merge_chunks(cache, output, total)?;
    info!("🎉 Final output at {}", output.display());

    Ok(())
}
