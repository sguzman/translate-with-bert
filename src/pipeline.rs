use anyhow::Result;
use log::{debug, info};
use rust_bert::pipelines::translation::{Language, TranslationModelBuilder};
use std::sync::OnceLock;

// Static model to avoid reloading in each thread
static MODEL: OnceLock<rust_bert::pipelines::translation::TranslationModel> = OnceLock::new();

fn get_model() -> &'static rust_bert::pipelines::translation::TranslationModel {
    MODEL.get_or_init(|| {
        info!("🔍 Loading Helsinki-NLP French → English translation model...");
        TranslationModelBuilder::new()
            .with_source_languages(vec![Language::French])
            .with_target_languages(vec![Language::English])
            .create_model()
            .expect("Failed to create translation model")
    })
}

pub fn translate_sentences(input: &[String]) -> Result<Vec<String>> {
    debug!("🔡 Translating chunk of size {}", input.len());
    let model = get_model();
    let output = model.translate(input, None, Language::French, Language::English)?;
    debug!("📤 Translation complete for {} sentence(s)", input.len());
    Ok(output)
}
