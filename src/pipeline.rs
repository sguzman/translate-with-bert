// src/pipeline.rs

use anyhow::Result;
use log::{debug, error, info};
use rust_bert::pipelines::translation::{
    Language,
    // You can swap in a larger checkpoint like m2m100_1.2B
    TranslationModel,
};
use std::cell::RefCell;
use tch::Device;

use rust_bert::m2m_100::{
    M2M100ConfigResources, M2M100MergesResources, M2M100ModelResources, M2M100VocabResources,
};
use rust_bert::pipelines::common::{ModelResource, ModelType};
use rust_bert::pipelines::translation::TranslationConfig;
use rust_bert::resources::RemoteResource;

thread_local! {
    /// One `TranslationModel` per thread.
    static THREAD_MODEL: RefCell<Option<TranslationModel>> = RefCell::new(None);
}

// Expect cuda to be available
pub fn cuda() -> Device {
    let device = Device::cuda_if_available();

    match device {
        Device::Cuda(_) => info!("🖥️  Using device: {:?}", device),
        Device::Cpu => {
            error!(
                "❌ No CUDA GPU available (detected {:?}). Aborting.",
                device
            );
            std::process::exit(1);
        }
        _ => info!("🖥️  Using device: {:?}", device),
    }
    device
}

pub fn build() -> TranslationModel {
    // 1. Weights & config for the XL (1.2 B) checkpoint
    let model_resource = ModelResource::Torch(Box::new(RemoteResource::from_pretrained(
        M2M100ModelResources::M2M100_1_2B,
    )));
    let config_resource = RemoteResource::from_pretrained(M2M100ConfigResources::M2M100_1_2B);

    // 2. ***This is the SentencePiece model, not vocab.json!***
    let vocab_resource = RemoteResource::from_pretrained(
        M2M100VocabResources::M2M100_1_2B, // points to `spiece.model`
    );

    let merges_resource = Some(RemoteResource::from_pretrained(
        M2M100MergesResources::M2M100_1_2B, // merges.txt
    ));

    // 2) M2M-100 can translate between ANY pair of 100 languages,
    //    so we list all of them once for src and once for tgt.
    let source_languages = vec![Language::French]; // or .iter().collect()
    let target_languages = vec![Language::English];
    let device = cuda();

    // 3) Build a TranslationConfig **and** tweak repetition knobs
    let mut cfg = TranslationConfig::new(
        ModelType::M2M100,
        model_resource,
        config_resource,
        vocab_resource,
        merges_resource,
        source_languages,
        target_languages,
        device,
    );

    // ---- anti-repetition tweaks ---------------------------------
    cfg.no_repeat_ngram_size = 3; // forbid repeating any trigram
    cfg.repetition_penalty = 1.15;
    cfg.num_beams = 5; // higher-quality decoding
    cfg.use_onnx = true;
    //----------------------------------------------------------------

    // 4) Instantiate the pipeline
    TranslationModel::new(cfg).unwrap()
}

/// Translate a batch of chunk-strings in one shot on the GPU.
/// Initializes the model once (on CUDA).
pub fn translate_chunks(inputs: &[String]) -> Result<Vec<String>> {
    debug!("🔡 Translating batch of {} chunk(s)", inputs.len());
    THREAD_MODEL.with(|cell| -> Result<Vec<String>> {
        if cell.borrow().is_none() {
            let model = build();
            // Use the larger 1.2B variant for better fluency:
            *cell.borrow_mut() = Some(model);
        }
        let binding = cell.borrow();
        let model = binding.as_ref().unwrap();
        let outputs = model.translate(inputs, Some(Language::French), Some(Language::English))?;
        debug!("✅ Batch translation completed");
        Ok(outputs)
    })
}
