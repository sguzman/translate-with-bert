// src/pipeline.rs

use anyhow::Result;
use log::{debug, info};
use rust_bert::pipelines::translation::{
    Language,
    // You can swap in a larger checkpoint like m2m100_1.2B
    TranslationModel,
    TranslationModelBuilder,
};
use std::cell::RefCell;
use tch::Device;

thread_local! {
    /// One `TranslationModel` per thread.
    static THREAD_MODEL: RefCell<Option<TranslationModel>> = RefCell::new(None);
}

/// Translate a batch of chunk-strings in one shot on the GPU.
/// Initializes the model once (on CUDA).
pub fn translate_chunks(inputs: &[String]) -> Result<Vec<String>> {
    debug!("🔡 Translating batch of {} chunk(s)", inputs.len());
    THREAD_MODEL.with(|cell| -> Result<Vec<String>> {
        if cell.borrow().is_none() {
            let device = Device::cuda_if_available();
            info!("🧵 Loading model on device: {:?}", device);
            let model = TranslationModelBuilder::new()
                // Use the larger 1.2B variant for better fluency:
                .with_xlarge_model()
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
