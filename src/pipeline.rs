use anyhow::Result;
use log::{debug, info};
use rust_bert::pipelines::translation::{Language, TranslationModel, TranslationModelBuilder};
use std::cell::RefCell;

thread_local! {
    /// One `TranslationModel` per worker thread.
    static THREAD_MODEL: RefCell<Option<TranslationModel>> = RefCell::new(None);
}

/// Translate a chunk (slice of sentences) in the current thread.
/// Initializes the model once, then reuses it for subsequent calls.
pub fn translate_chunk(input: &[String]) -> Result<Vec<String>> {
    debug!("🔡 Translating {} sentence(s)", input.len());
    THREAD_MODEL.with(|cell| -> Result<Vec<String>> {
        // Initialize on first use
        if cell.borrow().is_none() {
            info!("🧵 Loading translation model in this thread...");
            let model = TranslationModelBuilder::new()
                .with_source_languages(vec![Language::French])
                .with_target_languages(vec![Language::English])
                .create_model()
                .expect("Failed to initialize model");
            *cell.borrow_mut() = Some(model);
        }
        // Borrow the initialized model and run translation
        let binding = cell.borrow();
        let model = binding.as_ref().unwrap();
        let output = model.translate(input, Some(Language::French), Some(Language::English))?;
        debug!("✅ Chunk translated");
        Ok(output)
    })
}
