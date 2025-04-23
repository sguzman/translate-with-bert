use regex::Regex;

/// Splits a block of text into sentences.
pub fn split_into_sentences(text: &str) -> Vec<String> {
    let re = Regex::new(r"(?m)(.*?[\.\?!])\s+").unwrap();
    let mut sentences = Vec::new();
    let mut last_end = 0;

    for cap in re.captures_iter(text) {
        if let Some(m) = cap.get(1) {
            sentences.push(m.as_str().trim().to_string());
            last_end = m.end();
        }
    }

    if last_end < text.len() {
        sentences.push(text[last_end..].trim().to_string());
    }

    sentences.into_iter().filter(|s| !s.is_empty()).collect()
}
