mod cleanup;
mod io;
mod pipeline;

use anyhow::Result;
use colored::*;
use env_logger;
use log::info;

fn main() -> Result<()> {
    env_logger::init();
    info!("{}", "Starting French→English translator...".green());

    let input_path = "data/input.txt";
    let output_path = "data/output_en.txt";

    let raw_text = io::read_file(input_path)?;
    let cleaned = cleanup::clean_text(&raw_text);
    let translated = pipeline::translate_paragraphs(&cleaned)?;
    io::write_file(output_path, &translated)?;

    info!("{}", "Translation complete!".cyan());
    Ok(())
}
