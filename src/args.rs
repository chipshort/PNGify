use clap::{Parser, Subcommand};
use image::ImageFormat;
use std::path::PathBuf;

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

        /// The image format the data should be encoded as.
        /// If none is provided, but an output file is used, the file extension is used to guess it.
        /// If none is provided and stdout is used, png is assumed.
        #[clap(short, long, arg_enum)]
        format: Option<FileFormat>,
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

        /// The image format that should be decoded.
        /// If none is provided, but an input file is used, the file extension is used to guess it.
        /// If none is provided and stdin is used, png is assumed.
        #[clap(short, long, arg_enum)]
        format: Option<FileFormat>,
    },
}

#[derive(clap::ArgEnum, Clone)]
pub enum FileFormat {
    Png,
    Pgm,
}

impl TryFrom<ImageFormat> for FileFormat {
    type Error = ();

    fn try_from(value: ImageFormat) -> Result<Self, Self::Error> {
        match value {
            ImageFormat::Png => Ok(FileFormat::Png),
            ImageFormat::Pnm => Ok(FileFormat::Pgm),
            _ => Err(()),
        }
    }
}
