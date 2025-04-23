use anyhow::Result;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn read_file(path: &str) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}

pub fn write_file(path: &str, content: &Vec<String>) -> Result<()> {
    let joined = content.join("\n\n");
    fs::write(path, joined)?;
    Ok(())
}

pub fn merge_chunks(cache_dir: &str, output_path: &str) -> Result<()> {
    let mut files: Vec<_> = fs::read_dir(cache_dir)?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("english."))
        .collect();

    files.sort_by_key(|entry| entry.file_name());

    let mut output = BufWriter::new(File::create(output_path)?);
    for entry in files {
        let content = fs::read_to_string(entry.path())?;
        writeln!(output, "{}\n", content)?;
    }

    Ok(())
}
