# Translator‑CLI & Core

> **From X to Y in 100 languages — an open‑source, Rust‑powered path toward high‑quality, offline machine translation**

---

## 🚀 Project Vision

`translator-cli` and `translator-core` together form a minimal, extensible foundation for an **end‑to‑end multilingual translation pipeline**.  The long‑term objective is *fully‑automatic* translation between **any pair of \~100 languages** supported by modern multilingual language‑model families (e.g. M2M‑100, NLLB‑200, SeamlessM4T, Whisper, „MLLM“‑style mixtures).

Although the current proof‑of‑concept merely echoes input text (see *Status & Road‑map*), the scaffolding already enforces good engineering practice: builder pattern, explicit device selection, streaming batch translation, on‑disk incremental caching, robust CLI ergonomics, and a fast test‑suite.

---

## ✨ Features (Current & Planned)

| Category                | Current                                                                                                                         | Road‑map                                                                                                   |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| **CLI usability**       | `translator-cli` binary with `--src`, `--tgt`, `--input`, `--output`, `--batch`, `--model`, `--resume`, `--quiet/verbose` flags | Sub‑command hierarchy (`translate`, `list-langs`, `inspect-model`); progress bars; JSONL & streaming modes |
| **Language support**    | Hard‑wired defaults `fr→en`                                                                                                     | 100+ languages via HF `M2M‑100` / `NLLB` weights, auto‑detection, language aliases                         |
| **Model zoo**           | `Tiny`/`Small`/`Base`/`Large` *place‑holders*                                                                                   | Pluggable back‑ends (MarianMT, CTranslate2, GGUF, llama.cpp, ONNX, GPU‑accelerated Torch)                  |
| **Hardware**            | CPU only (`cuda_if_available()` returns `Cpu`)                                                                                  | CUDA, ROCm, Metal; batched GPU decoding; mixed‑precision                                                   |
| **Segmentation**        | Lightweight rule‑based sentence splitter                                                                                        | Unicode‑aware tokenizer; heuristics for abbreviations & quotes; optional language‑specific models          |
| **Caching**             | SHA‑256 hashed sentence chunks in `.cache/`                                                                                     | LM‑embeddings deduplication; RocksDB/SQLite cache; partial‑file checkpointing                              |
| **Testing**             | 8 unit tests (4 currently failing, see *Audit*)                                                                                 | CI matrix (stable/nightly, Linux/macOS/Windows); property‑based fuzzing; benchmark suite                   |
| **Logging & Telemetry** | `tracing` with env filter                                                                                                       | Structured JSON logs; OpenTelemetry exporter                                                               |
| **Packaging**           | Cargo workspace                                                                                                                 | Nix flake, Docker image, Home‑brew formula, pre‑compiled GitHub Releases                                   |

---

## 🏗️ Architecture Overview

```text
┌────────────────────────┐        ┌──────────────────────────────┐
│  translator-cli (bin)  │──API──▶│   translator-core (library)  │
└────────────────────────┘        │  • TranslatorBuilder         │
                                  │  • Translator                │
                                  │  • Device / ModelSize enums  │
                                  │  • Sentence segmentation     │
                                  │  • On‑disk cache             │
                                  └──────────────────────────────┘
```

* **`translator-core`** (crate)

  * Pure library — *no* `unsafe` blocks.
  * Re‑exported builder pattern ensures immutable configuration once built.
  * Caching layer maps each joined batch chunk → SHA‑256 filename for deterministic resume.
  * Naïve sentence splitter avoids additional deps; pluggable for spaCy, `rust-nlp`, etc.

* **`translator-cli`** (crate)

  * Thin wrapper around the core with [`clap`](https://docs.rs/clap) auto‑generated help.
  * Respects Unix streams but encourages file‑to‑file operation for large texts.
  * Verbosity negotiated via `tracing_subscriber` *env‑filter* (error | info | debug).

---

## 🔍 Program Audit (v0.1.0‑prototype)

| Area                   | Status                                             | Observations & Recommended Actions                                                                                               |
| ---------------------- | -------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| **Unit tests**         | **6 total**; *4 failing*                           | Tests expect real `fr→en` output ("Bonjour → Hello"). `translate` currently echoes input; implement inference or adapt fixtures. |
| **Error handling**     | ✅ uses `anyhow` for context‑rich errors            | Wrap `File` I/O in `BufReader/Writer` for large files; bubble up I/O size.                                                       |
| **Safety**             | ✅ no `unsafe` code, denies warnings                | Add `#![forbid(unsafe_code)]` for clarity.                                                                                       |
| **Performance**        | 🚫 dummy inference                                 | Integrate tokenizer + batched decoding; parallelise with `rayon` once CPU baseline works.                                        |
| **Device abstraction** | Stub                                               | Implement CUDA → CuBLAS / TensorRT path; runtime automatic fall‑back.                                                            |
| **Config persistence** | None                                               | Add `~/.config/translator/config.toml` support; allow env overrides.                                                             |
| **CLI UX**             | Good defaults; helpful flags                       | Add colored diagnostics via `miette` or `console`.                                                                               |
| **Logging**            | Minimum viable                                     | Emit progress (% translated); hide cache hits behind `debug`.                                                                    |
| **Security**           | Writes only inside user‑specified dirs; no net I/O | Verify cache path canonicalisation; sanitise language codes to prevent path traversal.                                           |

---

## 🛠️ Installation

```bash
# 1. Prerequisites
#    • Rust ≥ 1.76 (edition 2021)
#    • Git, CMake, Python3 (if building Torch‑based back‑ends later)
#    • Optional: CUDA ≥ 12 for GPU inference

# 2. Clone & build
$ git clone https://github.com/yourname/translator.git
$ cd translator
$ cargo build --release

# 3. Run tests
$ cargo test           # (expect failures until backend implemented)
```

A Dockerfile and Nix flake are planned; PRs welcome.

---

## 🚴‍♀️ Quick‑Start Usage

```bash
# French → English (CPU, default tiny model)
$ translator-cli --src fr --tgt en \
    --input examples/input.txt \
    --output out.txt

# Resume an interrupted run, with larger batch & model
$ translator-cli --src ja --tgt de \
    --batch 8 --model large --resume \
    --input novel.txt --output novel_de.txt
```

### Supported Language Codes

Run `translator-cli --list-langs` *(coming soon)* to print ISO‑639‑1/3 codes.  The target is **100+ languages** spanning Latin, Cyrillic, Arabic, CJK, Indic, RTL scripts, etc.

---

## 🧪 Testing & Continuous Integration

* `cargo test` covers segmentation, cache integrity, and *expected* translation output.
* Planned GitHub Actions matrix: stable/nightly, Ubuntu‑latest + macOS + Windows.
* Benchmarks via [`criterion`](https://github.com/bheisler/criterion.rs).

---

## 📈 Performance & Memory Targets *(post‑MVP)*

| Hardware            | Throughput (tok/s) | Latency (≤ 1k words) | Memory Footprint |
| ------------------- | ------------------ | -------------------- | ---------------- |
| Laptop CPU (4‑core) | ≥ 50               | ≤ 2 s                | < 2 GB RAM       |
| RTX‑3090            | ≥ 800              | ≤ 0.3 s              | < 8 GB VRAM      |

---

## 🤝 Contributing

1. Fork & create feature branches (`feat/…`, `fix/…`).
2. Run `cargo fmt` & `cargo clippy -- -D warnings`.
3. Write/adjust tests; ensure `cargo test` passes.
4. Open a PR; fill in the template.

Beginner‑friendly issues are tagged **good first issue**.

---

## 🗺️ Road‑map (Q3 – Q4 2025)

1. **Backend integration** — plug in HF `ct2` Marian/NLLB models (CPU & GPU).
2. **Dynamic language detection** — `whatlang`/`cld3` auto‑detect unless `--src` specified.
3. **Streaming & interactive mode** — translate stdin line‑by‑line with minimal latency.
4. **Advanced segmentation** — intl. punctuation, abbreviations, newline heuristics.
5. **Model management** — `translator models pull`, version pinning, checksums.
6. **WebAssembly target** — in‑browser demo using `wasm‑bindgen` + WebGPU.
7. **Docs & examples** — notebooks, blog posts, benchmark dashboard.
8. **v1.0** release — full test‑suite passes for 20 high‑traffic language pairs.

---

## 📜 License

This project is licensed under **CC0-1.0**.

```
SPDX-License-Identifier: CC0-1.0
```

---

## 📣 Acknowledgements

* HuggingFace Transformers & CTranslate2 for open model access.
* *tracing*, *anyhow*, and the wider Rust ecosystem for stellar DX.
* Early contributors and testers — your feedback steers the ship!

---

> *“Translators are the bearers of the future.”* — **Boris Pasternak**

