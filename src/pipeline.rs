use anyhow::Result;
use log::{debug, info};
use rust_bert::pipelines::translation::{Language, TranslationModel, TranslationModelBuilder};
use std::cell::RefCell;

thread_local! {
    /// Each worker thread gets its own `TranslationModel`.
    static THREAD_MODEL: RefCell<Option<TranslationModel>> = RefCell::new(None);
}

/// Initialize (once per thread) and clone the model for use.
fn get_model() -> TranslationModel {
    THREAD_MODEL.with(|cell| {
        if cell.borrow().is_none() {
            info!("🧵 Loading translation model in thread...");
            let model = TranslationModelBuilder::new()
                .with_source_languages(vec![Language::French])
                .with_target_languages(vec![Language::English])
                .create_model()
                .expect("Failed to load model");
            *cell.borrow_mut() = Some(model);
        }
        // Clone the model handle for this call
        cell.borrow().as_ref().unwrap().clone()
    })
}

/// Translate one chunk (a slice of sentences) and return the English sentences.
pub fn translate_chunk(input: &[String]) -> Result<Vec<String>> {
    debug!("Translating {} sentence(s)", input.len());
    let model = get_model();
    // API takes: (inputs, source_lang, target_lang)
    let output = model.translate(input, Some(Language::French), Some(Language::English))?;
    debug!("Translation done");
    Ok(output)
}
