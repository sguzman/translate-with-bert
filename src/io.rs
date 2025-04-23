use anyhow::Result;
use colored::*;
use log::info;
use regex::Regex;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

/// Split full text into sentences (keeps delimiters: ., ?, !).
pub fn split_into_sentences(text: &str) -> Vec<String> {
    let re = Regex::new(r"(?m)(.*?[\.\?!])\s+").unwrap();
    let mut sentences = Vec::new();
    let mut last = 0;
    for cap in re.captures_iter(text) {
        if let Some(m) = cap.get(1) {
            sentences.push(m.as_str().trim().to_string());
            last = m.end();
        }
    }
    if last < text.len() {
        sentences.push(text[last..].trim().to_string());
    }
    sentences.into_iter().filter(|s| !s.is_empty()).collect()
}

/// Write a chunk’s lines to a file.
pub fn write_chunk(path: &Path, lines: &[String]) -> Result<()> {
    let mut f = File::create(path)?;
    for line in lines {
        writeln!(f, "{}", line)?;
    }
    info!(
        "💾 Wrote {} lines to {}",
        lines.len().to_string().green(),
        path.display()
    );
    Ok(())
}

/// Merge all numbered chunk files in `cache_dir` into `output`, in order.
pub fn merge_chunks(cache_dir: &Path, output: &Path, total: usize) -> Result<()> {
    let mut out = File::create(output)?;
    for i in 0..total {
        let p = cache_dir.join(format!("english.{:02}.txt", i));
        if p.exists() {
            let text = fs::read_to_string(&p)?;
            writeln!(out, "{}\n", text)?;
        }
    }
    info!("📦 Merged {} chunks into {}", total, output.display());
    Ok(())
}
