pub fn clean_text(raw: &str) -> Vec<String> {
    let mut paragraphs = vec![];

    for para in raw.split("\n\n") {
        let line = para.trim().replace("\n", " ");
        if !line.is_empty() {
            paragraphs.push(line.to_string());
        }
    }

    paragraphs
}
