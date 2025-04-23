use anyhow::Result;
use log::{debug, info};
use rust_bert::pipelines::translation::{Language, TranslationModel, TranslationModelBuilder};
use std::cell::RefCell;

thread_local! {
    /// One model per worker thread.
    static THREAD_MODEL: RefCell<Option<TranslationModel>> = RefCell::new(None);
}

fn get_model() -> TranslationModel {
    THREAD_MODEL.with(|cell| {
        if cell.borrow().is_none() {
            info!("🧵 Loading translation model in this thread...");
            let model = TranslationModelBuilder::new()
                .with_source_languages(vec![Language::French])
                .with_target_languages(vec![Language::English])
                .create_model()
                .expect("Failed to initialize model");
            *cell.borrow_mut() = Some(model);
        }
        // Dereference before clone to get owned TranslationModel
        (*cell.borrow().as_ref().unwrap()).clone()
    })
}

/// Translate one chunk (slice of sentences).
pub fn translate_chunk(input: &[String]) -> Result<Vec<String>> {
    debug!("🔡 Translating {} sentence(s)", input.len());
    let model = get_model();
    let out = model.translate(input, Some(Language::French), Some(Language::English))?;
    debug!("✅ Chunk translated");
    Ok(out)
}
