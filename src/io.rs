use anyhow::Result;
use std::fs;

pub fn read_file(path: &str) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}

pub fn write_file(path: &str, content: &Vec<String>) -> Result<()> {
    let joined = content.join("\n\n");
    fs::write(path, joined)?;
    Ok(())
}
