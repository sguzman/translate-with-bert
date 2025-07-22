#![deny(warnings)]

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Device to run translation on.
#[derive(Debug, Clone, Copy)]
pub enum Device {
    Cpu,
    Cuda,
}

impl Device {
    pub fn cuda_if_available() -> Self {
        // In this lightweight example we always return CPU.
        Self::Cpu
    }
}

/// Model size selection.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum ModelSize {
    Tiny,
    Small,
    Base,
    Large,
}

/// Builder for [`Translator`].
pub struct TranslatorBuilder {
    device: Device,
    source: String,
    target: String,
    model: ModelSize,
    batch: usize,
    cache_dir: PathBuf,
}

impl TranslatorBuilder {
    pub fn new() -> Self {
        Self {
            device: Device::cuda_if_available(),
            source: "fr".into(),
            target: "en".into(),
            model: ModelSize::Tiny,
            batch: 4,
            cache_dir: PathBuf::from(".cache"),
        }
    }

    pub fn device(mut self, d: Device) -> Self {
        self.device = d;
        self
    }

    pub fn source<S: Into<String>>(mut self, s: S) -> Self {
        self.source = s.into();
        self
    }

    pub fn target<S: Into<String>>(mut self, s: S) -> Self {
        self.target = s.into();
        self
    }

    pub fn model_size(mut self, m: ModelSize) -> Self {
        self.model = m;
        self
    }

    pub fn batch_size(mut self, b: usize) -> Self {
        self.batch = b;
        self
    }

    pub fn cache_dir<P: Into<PathBuf>>(mut self, p: P) -> Self {
        self.cache_dir = p.into();
        self
    }

    pub fn build(self) -> Result<Translator> {
        fs::create_dir_all(&self.cache_dir).with_context(|| "create cache dir")?;
        Ok(Translator {
            device: self.device,
            source: self.source,
            target: self.target,
            model: self.model,
            batch: self.batch,
            cache_dir: self.cache_dir,
        })
    }
}

#[allow(dead_code)]
pub struct Translator {
    device: Device,
    source: String,
    target: String,
    model: ModelSize,
    batch: usize,
    cache_dir: PathBuf,
}

impl Translator {
    pub fn builder() -> TranslatorBuilder {
        TranslatorBuilder::new()
    }

    pub fn translate(&self, sentences: &[String]) -> Result<Vec<String>> {
        // Dummy translator: simply echoes input.
        Ok(sentences.iter().map(|s| s.to_string()).collect())
    }

    pub fn segment(&self, text: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut start = 0;
        for (idx, c) in text.char_indices() {
            if matches!(c, '.' | '!' | '?') {
                let slice = &text[start..=idx];
                out.push(slice.trim().to_string());
                start = idx + 1;
            }
        }
        if start < text.len() {
            out.push(text[start..].trim().to_string());
        }
        out.retain(|s| !s.is_empty());
        out
    }

    pub fn cache_path(&self, text: &str) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        self.cache_dir.join(format!("{hash}.txt"))
    }

    pub fn translate_file(&self, input: &Path, output: &Path, resume: bool) -> Result<()> {
        let mut buf = String::new();
        File::open(input)
            .with_context(|| format!("open {}", input.display()))?
            .read_to_string(&mut buf)
            .with_context(|| format!("read {}", input.display()))?;

        let sents = self.segment(&buf);

        let mut results = Vec::new();
        for (i, chunk_slice) in sents.chunks(self.batch).enumerate() {
            let chunk = chunk_slice.join(" ");
            let path = self.cache_path(&chunk);
            if resume && path.exists() {
                let mut cached = String::new();
                File::open(&path).unwrap().read_to_string(&mut cached).unwrap();
                results.push((i, cached));
            } else {
                let translated = self.translate(&[chunk.clone()])?.pop().unwrap();
                File::create(&path).unwrap().write_all(translated.as_bytes()).unwrap();
                results.push((i, translated));
            }
        }

        results.sort_by_key(|(i, _)| *i);
        let mut file = File::create(output)?;
        for (_, line) in results {
            writeln!(file, "{}", line)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_defaults() {
        let t = Translator::builder().build().unwrap();
        assert_eq!(t.batch, 4);
    }

    #[test]
    fn cache_path_hash() {
        let t = Translator::builder().cache_dir("tmp").build().unwrap();
        let p = t.cache_path("hello");
        assert!(p.to_str().unwrap().contains("tmp"));
    }
}
