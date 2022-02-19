use anyhow::anyhow;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::vec::Vec;

use clap::Parser;
use derive_enum_from_into::EnumFrom;
use image::{ImageDecoder, ImageEncoder};
use match_any::match_any;

use crate::args::{Cli, Command, FileFormat};
use crate::bytes::Bytes;
use crate::bytes::U64_BYTES;

mod args;
mod bytes;

#[derive(EnumFrom)]
enum InputReader {
    Stdin(io::Stdin),
    File(std::fs::File),
}

#[derive(EnumFrom)]
enum OutputWriter {
    Stdout(io::Stdout),
    File(std::fs::File),
}

fn main() {
    let args: Cli = Cli::parse();

    match args.command {
        Command::Encode {
            input,
            output,
            format,
        } => {
            let data = match input {
                None => read_stdin(),
                Some(path) => read_file(path),
            };

            let format = determine_format(&output, format);

            match (data, open_output(output)) {
                (Ok(input), Ok(output)) => {
                    use OutputWriter::*;
                    let result =
                        match_any!(output, Stdout(o) | File(o) => encode_data(input, o, format));
                    if let Err(e) = result {
                        eprintln!("Error encoding the image: {}", e);
                    }
                }
                (Err(e), _) => {
                    eprintln!("Error reading input: {}", e);
                }
                (_, Err(e)) => {
                    eprintln!("Error writing output: {}", e);
                }
            }
        }
        Command::Decode {
            input,
            output,
            format,
        } => {
            let format = determine_format(&input, format);

            match (open_input(input), open_output(output)) {
                (Ok(input), Ok(output)) => {
                    use InputReader as In;
                    use OutputWriter as Out;
                    let result = match_any!(input, In::Stdin(i) | In::File(i) => {
                        match_any!(output, Out::Stdout(o) | Out::File(o) => decode_data(i, o, format))
                    });
                    if let Err(e) = result {
                        eprintln!("Error decoding the image: {}", e);
                    }
                }
                (Err(e), _) => {
                    eprintln!("Error reading input: {:?}", e);
                }
                (_, Err(e)) => {
                    eprintln!("Error writing output: {:?}", e);
                }
            }
        }
    }
}

/// Determines the image format using the following order (falling priority): provided_format, path, assume FileFormat::Png
fn determine_format(path: &Option<PathBuf>, provided_format: Option<FileFormat>) -> FileFormat {
    // try to guess from path
    let format = match path {
        None => None,
        Some(path) => provided_format.or(image::ImageFormat::from_path(path)
            .ok()
            .map(|f| f.try_into().ok())
            .flatten()),
    };
    // use png as fallback
    format.unwrap_or(FileFormat::Png)
}

/// Opens the file and reads it's contents into a Vec
fn read_file(path: PathBuf) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let file_size = file.metadata()?.len();

    // can use file size as a capacity hint for the Vec
    let mut data = Vec::with_capacity(file_size as usize);
    file.read_to_end(&mut data)?;

    Ok(data)
}

fn read_stdin() -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    io::stdin().read_to_end(&mut data)?;

    Ok(data)
}

fn open_input(file: Option<PathBuf>) -> io::Result<InputReader> {
    match file {
        None => Ok(io::stdin().into()),
        Some(path) => Ok(File::open(path)?.into()),
    }
}

fn open_output(file: Option<PathBuf>) -> io::Result<OutputWriter> {
    match file {
        None => Ok(io::stdout().into()),
        Some(path) => Ok(std::fs::File::create(path)?.into()),
    }
}

fn encode_data(mut data: Vec<u8>, out: impl Write, format: FileFormat) -> anyhow::Result<()> {
    let data_size = data.len();
    // we could use prime factorization to get a rect that fits exactly, but that's a lot of work and
    // will result in very weird dimensions for prime numbers, so it is probably not worth the hassle,
    // so instead, we just use a square that's just big enough and fill the rest with zeroes
    let image_dimension = ((data_size + U64_BYTES) as f32).sqrt().ceil() as usize;
    data.resize((image_dimension * image_dimension) as usize, 0);
    // write data size at the end of the image
    let pos = data.len() - U64_BYTES;
    data.write_u64(pos, data_size as u64);

    // encode as given format
    let writer = io::BufWriter::new(out);
    match format {
        FileFormat::Png => image::codecs::png::PngEncoder::new(writer).write_image(
            &data,
            image_dimension as u32,
            image_dimension as u32,
            image::ColorType::L8,
        )?,
        FileFormat::Pgm => image::codecs::pnm::PnmEncoder::new(writer).write_image(
            &data,
            image_dimension as u32,
            image_dimension as u32,
            image::ColorType::L8,
        )?,
    }

    Ok(())
}

fn decode_data(reader: impl Read, out: impl Write, format: FileFormat) -> anyhow::Result<()> {
    let reader = io::BufReader::new(reader);

    // TODO: autodetect format from file instead of defaulting to png if stdin is used
    // format = image::guess_format()
    // decode image as given format
    let data = match format {
        FileFormat::Png => {
            let decoder = image::codecs::png::PngDecoder::new(reader)?;
            let mut data = vec![0; decoder.total_bytes() as usize];
            decoder.read_image(&mut data)?;
            data
        }
        FileFormat::Pgm => {
            let decoder = image::codecs::pnm::PnmDecoder::new(reader)?;
            let mut data = vec![0; decoder.total_bytes() as usize];
            decoder.read_image(&mut data)?;
            data
        }
    };

    let original_len_index = data.len() - U64_BYTES;
    // get original length
    let original_len = data.read_u64(original_len_index);

    // check bounds
    if original_len > original_len_index as u64 {
        return Err(anyhow!("Invalid image file, original length is too big"));
    }

    // write file to out
    let mut out = io::BufWriter::new(out);
    out.write_all(&data[0..original_len as usize])?;
    out.flush()?;

    Ok(())
}
