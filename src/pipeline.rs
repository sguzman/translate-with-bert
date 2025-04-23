use anyhow::Result;
use log::debug;
use rust_bert::pipelines::translation::{Language, TranslationModelBuilder};

pub fn translate_paragraphs(paragraphs: &[String]) -> Result<Vec<String>> {
    let model = TranslationModelBuilder::new()
        .with_source_languages(vec![Language::French])
        .with_target_languages(vec![Language::English])
        .create_model()?;

    let mut results = vec![];
    for chunk in paragraphs.chunks(4) {
        debug!("Translating chunk of size {}", chunk.len());
        let translations =
            model.translate(chunk, Some(Language::French), Some(Language::English))?;

        results.extend(translations);
    }

    debug!("Total translations made: {}", results.len());

    Ok(results)
}
