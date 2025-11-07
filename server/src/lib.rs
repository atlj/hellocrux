pub mod media;
pub mod prepare;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Clone)]
#[command()]
pub struct Args {
    /// Path to the media dir
    #[arg(short, long, default_value = "./media")]
    pub media_dir: PathBuf,
}
