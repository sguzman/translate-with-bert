use crate::io::{read_chunk, write_chunk};
use anyhow::Result;
use colored::*;
use log::{info, warn};
use rayon::prelude::*;
use rust_bert::pipelines::translation::{Language, TranslationModel, TranslationModelBuilder};
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};

thread_local! {
    static THREAD_MODEL: RefCell<Option<TranslationModel>> = RefCell::new(None);
}

fn get_model() -> TranslationModel {
    THREAD_MODEL.with(|cell| {
        if cell.borrow().is_none() {
            info!("🧵 Loading translation model in thread...");
            let model = TranslationModelBuilder::new()
                .with_source_languages(vec![Language::French])
                .create_model()
                .expect("Failed to load model");
            *cell.borrow_mut() = Some(model);
        }
        cell.borrow().as_ref().unwrap().clone()
    })
}

fn list_unfinished_chunks(cache_dir: &Path, total_chunks: usize) -> Vec<usize> {
    let mut unfinished = Vec::new();
    for i in 0..total_chunks {
        let chunk_path = cache_dir.join(format!("english.{:02}.txt", i));
        if !chunk_path.exists() {
            unfinished.push(i);
        }
    }
    unfinished
}

pub fn run_translation(
    input_chunks: Vec<PathBuf>,
    cache_dir: &Path,
    output_dir: &Path,
) -> Result<()> {
    fs::create_dir_all(cache_dir)?;

    let unfinished = list_unfinished_chunks(cache_dir, input_chunks.len());
    info!("🔍 Unfinished chunks: {:?}", unfinished);

    unfinished.par_iter().for_each(|&i| {
        let input_path = &input_chunks[i];
        let cache_path = cache_dir.join(format!("english.{:02}.txt", i));

        match read_chunk(input_path) {
            Ok(lines) => {
                let model = get_model();
                let result = model.translate(&lines, None, Language::French);
                match result {
                    Ok(translations) => {
                        if let Err(e) = write_chunk(&cache_path, &translations) {
                            warn!("⚠️ Failed to write chunk {}: {}", i, e.to_string().red());
                        } else {
                            info!("✅ Finished chunk {}", i.to_string().green());
                        }
                    }
                    Err(e) => warn!(
                        "⚠️ Translation failed for chunk {}: {}",
                        i,
                        e.to_string().red()
                    ),
                }
            }
            Err(e) => warn!(
                "⚠️ Could not read input chunk {}: {}",
                i,
                e.to_string().red()
            ),
        }
    });

    let final_output_path = output_dir.join("english.txt");
    let mut all_lines = Vec::new();
    for i in 0..input_chunks.len() {
        let chunk_path = cache_dir.join(format!("english.{:02}.txt", i));
        if let Ok(lines) = read_chunk(&chunk_path) {
            all_lines.extend(lines);
        }
    }
    write_chunk(&final_output_path, &all_lines)?;

    info!(
        "🎉 Final file assembled at {}",
        final_output_path.display().to_string().bold()
    );
    Ok(())
}
