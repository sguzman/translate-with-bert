mod io;
mod pipeline;

use colored::*;
use env_logger::Env;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info};
use rayon::prelude::*;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    time::Instant,
};

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("{}", "🚀 Starting French→English translator...".bold());

    let input_path = "res/french.txt";
    let output_dir = ".cache";
    let output_final = "res/english.txt";

    fs::create_dir_all(output_dir)?;
    info!("📂 Ensured cache directory exists: {}", output_dir.cyan());

    let text = fs::read_to_string(input_path)?;
    let sentences = io::split_into_sentences(&text);
    let chunk_size = 4;
    let chunks: Vec<_> = sentences.chunks(chunk_size).enumerate().collect();

    let pb = ProgressBar::new(chunks.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} Chunks",
        )?
        .progress_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );

    let start = Instant::now();

    chunks.par_iter().for_each(|(i, chunk)| {
        let out_path = format!("{}/english.{:02}.txt", output_dir, i);
        if Path::new(&out_path).exists() {
            info!("⏭️  Chunk {:02} already cached", i);
            pb.inc(1);
            return;
        }

        match pipeline::translate_sentences(chunk) {
            Ok(translations) => {
                if let Ok(mut f) = File::create(&out_path) {
                    let _ = writeln!(f, "{}", translations.join("\n"));
                    info!("✅ Saved chunk {:02}", i);
                }
            }
            Err(e) => error!("❌ Chunk {:02} failed: {:?}", i, e),
        }
        pb.inc(1);
    });

    pb.finish_with_message("✨ All chunks processed.");

    // Merge chunks
    info!("{}", "📦 Merging all chunks...".bold());
    let mut final_out = File::create(output_final)?;
    for i in 0..chunks.len() {
        let part_path = format!("{}/english.{:02}.txt", output_dir, i);
        if let Ok(txt) = fs::read_to_string(&part_path) {
            writeln!(final_out, "{}", txt)?;
        }
    }

    info!("🎉 Finished in {:.2?}", start.elapsed());
    Ok(())
}
