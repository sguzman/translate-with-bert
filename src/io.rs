use anyhow::Result;
use colored::*;
use log::info;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
        info!(
            "📁 Created directory: {}",
            path.display().to_string().yellow()
        );
    }
    Ok(())
}

pub fn read_chunk(file_path: &PathBuf) -> Result<Vec<String>> {
    info!(
        "📥 Reading chunk: {}",
        file_path.display().to_string().cyan()
    );
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();
    info!(
        "📄 Read {} lines from {}",
        lines.len().to_string().blue(),
        file_path.display().to_string().cyan()
    );
    Ok(lines)
}

pub fn write_chunk(file_path: &PathBuf, lines: &[String]) -> Result<()> {
    let mut file = File::create(file_path)?;
    for line in lines {
        writeln!(file, "{}", line)?;
    }
    info!(
        "💾 Wrote {} lines to {}",
        lines.len().to_string().green(),
        file_path.display().to_string().green()
    );
    Ok(())
}
