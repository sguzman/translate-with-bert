use anyhow::Result;
use log::{debug, info};
use rust_bert::pipelines::translation::{Language, TranslationModel, TranslationModelBuilder};
use std::cell::RefCell;
use tch::Device;

thread_local! {
    /// One `TranslationModel` per thread (here: main thread).
    static THREAD_MODEL: RefCell<Option<TranslationModel>> = RefCell::new(None);
}

/// Translate a batch of “chunk-texts” all at once on GPU (if available).
/// Each input string is one chunk (e.g. 4 sentences joined by spaces).
pub fn translate_chunks(inputs: &[String]) -> Result<Vec<String>> {
    debug!("🔡 Batch-translating {} chunk(s)", inputs.len());
    THREAD_MODEL.with(|cell| -> Result<Vec<String>> {
        if cell.borrow().is_none() {
            let device = Device::cuda_if_available();
            info!("🧵 Loading model on device: {:?}", device);
            let model = TranslationModelBuilder::new()
                .with_device(device)
                .with_source_languages(vec![Language::French])
                .with_target_languages(vec![Language::English])
                .create_model()
                .expect("Failed to initialize model");
            *cell.borrow_mut() = Some(model);
        }
        let binding = cell.borrow();
        let model = binding.as_ref().unwrap();
        let outputs = model.translate(inputs, Some(Language::French), Some(Language::English))?;
        debug!("✅ Batch translation done");
        Ok(outputs)
    })
}
