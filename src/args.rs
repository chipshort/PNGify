use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Encode {
        /// The file to encode as an image.
        /// If none is provided, stdin is used.
        #[clap(short, long)]
        input: Option<PathBuf>,

        /// The image file to write to.
        /// If none is provided, stdout is used.
        #[clap(short, long)]
        output: Option<PathBuf>,

        // format: FileFormatEnum

        // size: Option<width or height of u64>
    },
    Decode {
        /// The image file to decode.
        /// If none is provided, stdin is used.
        #[clap(short, long)]
        input: Option<PathBuf>,

        /// The file to write to.
        /// If none is provided, stdout is used.
        #[clap(short, long)]
        output: Option<PathBuf>,

        // format: Option<FileFormatEnum> // only required if not clear by file extension (e.g. if stdin is used)
    },
}