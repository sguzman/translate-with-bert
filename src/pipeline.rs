use anyhow::Result;
use log::{debug, info};
use rust_bert::pipelines::translation::{Language, TranslationModel, TranslationModelBuilder};
use std::cell::RefCell;
use tch::Device;

thread_local! {
    /// One TranslationModel per thread (here: main thread)
    static THREAD_MODEL: RefCell<Option<TranslationModel>> = RefCell::new(None);
}

/// Translate a chunk of sentences, loading the model (on GPU if available) only once.
pub fn translate_chunk(input: &[String]) -> Result<Vec<String>> {
    debug!("🔡 Translating {} sentence(s)", input.len());
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
        let model = cell.borrow().as_ref().unwrap();
        let output = model.translate(input, Some(Language::French), Some(Language::English))?;
        debug!("✅ Chunk translated");
        Ok(output)
    })
}
