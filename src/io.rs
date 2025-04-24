// src/io.rs

use anyhow::Result;
use colored::*;
use log::{debug, info};
use regex::Regex;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use crate::cleanup;

/// Split full text into paragraphs (cleaned).
pub fn split_into_paragraphs(text: &str) -> Vec<String> {
    cleanup::clean_text(text)
}

/// Split full text into sentences (keeping the punctuation).
pub fn split_into_sentences(text: &str) -> Vec<String> {
    let re = Regex::new(r"(?m)(.*?[\.\?!])\s+").unwrap();
    let mut v = Vec::new();
    let mut last = 0;
    for cap in re.captures_iter(text) {
        if let Some(m) = cap.get(1) {
            v.push(m.as_str().trim().to_string());
            last = m.end();
        }
    }
    if last < text.len() {
        v.push(text[last..].trim().to_string());
    }
    v.into_iter().filter(|s| !s.is_empty()).collect()
}

/// Write a chunk (lines) to disk.
pub fn write_chunk(path: &Path, lines: &[String]) -> Result<()> {
    let mut f = File::create(path)?;
    for line in lines {
        writeln!(f, "{}", line)?;
    }
    debug!(
        "💾 Wrote {} lines to {}",
        lines.len().to_string().green(),
        path.display()
    );
    Ok(())
}

/// Merge `english.00.txt`…`english.NN.txt` into one file.
pub fn merge_chunks(cache_dir: &Path, out: &Path, total: usize) -> Result<()> {
    let mut f = File::create(out)?;
    for i in 0..total {
        let part = cache_dir.join(format!("english.{:02}.txt", i));
        if part.exists() {
            let mut buf = String::new();
            File::open(&part)?.read_to_string(&mut buf)?;
            writeln!(f, "{}\n", buf)?;
        }
    }
    info!(
        "📦 Merged {} chunks into {}",
        total.to_string().blue(),
        out.display()
    );
    Ok(())
}
