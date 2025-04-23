use anyhow::Result;
use log::{debug, info};
use rust_bert::pipelines::translation::{Language, TranslationModel, TranslationModelBuilder};
use std::cell::RefCell;
use tch::Device;

thread_local! {
    /// One `TranslationModel` per thread (main thread here).
    static THREAD_MODEL: RefCell<Option<TranslationModel>> = RefCell::new(None);
}

/// Translate a batch of chunk-texts (each chunk is a single String)
/// in one shot on the GPU. Initializes the model (on CUDA) once.
pub fn translate_chunks(inputs: &[String]) -> Result<Vec<String>> {
    debug!("🔡 Translating batch of {} chunk(s)", inputs.len());
    THREAD_MODEL.with(|cell| -> Result<Vec<String>> {
        if cell.borrow().is_none() {
            let device = Device::cuda_if_available();
            info!("🧵 Loading model on device: {:?}", device);
            let model = TranslationModelBuilder::new()
                .with_device(device)
                .with_source_languages(vec![Language::French])
                .with_target_languages(vec![Language::English])
                .create_model()?;
            *cell.borrow_mut() = Some(model);
        }
        let binding = cell.borrow();
        let model = binding.as_ref().unwrap();
        let outputs = model.translate(inputs, Some(Language::French), Some(Language::English))?;
        debug!("✅ Batch translation completed");
        Ok(outputs)
    })
}
