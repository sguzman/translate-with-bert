use anyhow::Result;
use std::fs;
use tempfile::tempdir;
use translator_core::Translator;

#[test]
fn basic_sentence() -> Result<()> {
    // Arrange
    let tmp = tempdir()?;
    let tr = Translator::builder()
        .source("fr")
        .target("en")
        .cache_dir(tmp.path())
        .build()?;
    // Act
    let out = tr.translate(&["Bonjour le monde !".to_string()])?.pop().unwrap();
    // Assert
    assert_eq!(out, "Hello world!");
    Ok(())
}

#[test]
fn mixed_punctuation() -> Result<()> {
    // Arrange
    let tmp = tempdir()?;
    let tr = Translator::builder()
        .source("fr")
        .target("en")
        .cache_dir(tmp.path())
        .build()?;
    // Act
    let out = tr.translate(&["¿Bonjour!? Quoi?".to_string()])?.pop().unwrap();
    // Assert
    assert_eq!(out, "¿Bonjour!? Quoi?");
    Ok(())
}

#[test]
fn diacritics() -> Result<()> {
    // Arrange
    let tmp = tempdir()?;
    let tr = Translator::builder()
        .source("fr")
        .target("en")
        .cache_dir(tmp.path())
        .build()?;
    // Act
    let out = tr.translate(&["Ça va très bien.".to_string()])?.pop().unwrap();
    // Assert
    assert_eq!(out, "I'm very well.");
    Ok(())
}

#[test]
fn multi_sentence_paragraph() -> Result<()> {
    // Arrange
    let tmp = tempdir()?;
    let tr = Translator::builder()
        .source("fr")
        .target("en")
        .cache_dir(tmp.path())
        .build()?;
    let paragraph = "Bonjour. Comment ça va? Très bien!";
    // Act
    let segments = tr.segment(paragraph);
    assert_eq!(segments, vec!["Bonjour.", "Comment ça va?", "Très bien!"]);
    let out = tr.translate(&[paragraph.to_string()])?.pop().unwrap();
    // Assert
    assert_eq!(out, "Hello. How are you? Very well!");
    Ok(())
}

#[test]
fn cache_resume() -> Result<()> {
    // Arrange
    let tmp = tempdir()?;
    let tr = Translator::builder()
        .source("fr")
        .target("en")
        .cache_dir(tmp.path())
        .build()?;
    let input_path = tmp.path().join("input.txt");
    let output_path = tmp.path().join("output.txt");
    fs::write(&input_path, "Bonjour")?;
    // Act
    tr.translate_file(&input_path, &output_path, true)?;
    tr.translate_file(&input_path, &output_path, true)?;
    // Assert
    assert!(output_path.exists());
    Ok(())
}

#[test]
fn edge_whitespace() -> Result<()> {
    // Arrange
    let tmp = tempdir()?;
    let tr = Translator::builder()
        .source("fr")
        .target("en")
        .cache_dir(tmp.path())
        .build()?;
    // Act
    let out = tr.translate(&["  Bonjour le monde !  \n".to_string()])?.pop().unwrap();
    // Assert
    assert_eq!(out, "Hello world!");
    Ok(())
}
