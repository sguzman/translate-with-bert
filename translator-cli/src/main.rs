use anyhow::Result;
use clap::{ArgAction, Parser};
use translator_core::{Device, ModelSize, Translator};
use tracing_subscriber::EnvFilter;
use std::path::PathBuf;

/// Simple translator CLI
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Source language code
    #[arg(long, default_value="fr")]
    src: String,
    /// Target language code
    #[arg(long, default_value="en")]
    tgt: String,
    /// Input file (defaults to stdin)
    #[arg(long)]
    input: Option<PathBuf>,
    /// Output file (defaults to stdout)
    #[arg(long)]
    output: Option<PathBuf>,
    /// Batch size
    #[arg(long, default_value_t = 4)]
    batch: usize,
    /// Model size
    #[arg(long, value_enum, default_value_t = ModelSize::Tiny)]
    model: ModelSize,
    /// Cache directory
    #[arg(long, default_value=".cache")]
    cache_dir: PathBuf,
    /// Resume from cache
    #[arg(long, action=ArgAction::SetTrue)]
    resume: bool,
    /// Quiet mode
    #[arg(long, action=ArgAction::SetTrue)]
    quiet: bool,
    /// Verbose mode
    #[arg(long, action=ArgAction::SetTrue)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let level = if args.quiet { "error" } else if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt().with_env_filter(EnvFilter::new(level)).init();

    let translator = Translator::builder()
        .device(Device::cuda_if_available())
        .source(args.src)
        .target(args.tgt)
        .model_size(args.model)
        .batch_size(args.batch)
        .cache_dir(args.cache_dir)
        .build()?;

    match (args.input, args.output) {
        (Some(i), Some(o)) => translator.translate_file(&i, &o, args.resume)?,
        _ => {
            eprintln!("input and output files required in this example");
        }
    }
    Ok(())
}
