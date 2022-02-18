use std::fs::File;
use std::io;
use std::io::{Read, Seek};
use std::io::Write;
use std::path::PathBuf;
use std::vec::Vec;

use clap::Parser;
use derive_enum_from_into::EnumFrom;
use image::{GrayImage, ImageDecoder, ImageEncoder, Luma};
use match_any::match_any;

use crate::bytes::Bytes;
use crate::bytes::U64_BYTES;
use crate::args::{Cli, Command};

mod bytes;
mod args;

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
        Command::Encode { input, output } => {
            let data = match input {
                None => read_stdin(),
                Some(path) => read_file(path)
            };

            match (data, open_output(output)) {
                (Ok(input), Ok(output)) => {
                    // TODO: error handling
                    use OutputWriter::*;
                    match_any!(output, Stdout(o) | File(o) => encode_data(input, o)).unwrap();
                }
                (Err(e), _) => {
                    eprintln!("Error reading input: {:?}", e);
                }
                (_, Err(e)) => {
                    eprintln!("Error writing output: {:?}", e);
                }
            }
        }
        Command::Decode { input, output } => {
            match (open_input(input), open_output(output)) {
                (Ok(input), Ok(output)) => {
                    use OutputWriter as Out;
                    use InputReader as In;
                    match_any!(input, In::Stdin(i) | In::File(i) => {
                        match_any!(output, Out::Stdout(o) | Out::File(o) => decode_data(i, o))
                    }).unwrap(); // TODO: error handling
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
        Some(path) => Ok(File::open(path)?.into())
    }
}

fn open_output(file: Option<PathBuf>) -> io::Result<OutputWriter> {
    match file {
        None => Ok(io::stdout().into()),
        Some(path) => Ok(std::fs::File::create(path)?.into()),
    }
}

fn encode_data(mut data: Vec<u8>, out: impl Write) -> anyhow::Result<()> {
    let data_size = data.len();
    // we could use prime factorization to get a rect that fits exactly, but that's a lot of work and
    // will result in very weird dimensions for prime numbers, so it is probably not worth the hassle,
    // so instead, we just use a square that's just big enough and fill the rest with zeroes
    let image_dimension = ((data_size + U64_BYTES) as f32).sqrt().ceil() as u32;
    data.resize((image_dimension * image_dimension) as usize, 0); // TODO: support random values?
    // write data size at the end of the image
    let pos = data.len() - U64_BYTES;
    data.write_u64(pos, data_size as u64);

    //Encode as png
    let mut writer = io::BufWriter::new(out);
    image::codecs::png::PngEncoder::new(writer).write_image(&data, image_dimension as u32, image_dimension as u32, image::ColorType::L8)?;

    Ok(())
}

fn decode_data(reader: impl Read, out: impl Write) -> anyhow::Result<()> {
    // let image = imagefmt::read_from(&mut reader, imagefmt::ColFmt::Y)?;
    let reader = io::BufReader::new(reader);

    let decoder = image::codecs::png::PngDecoder::new(reader)?; // TODO: allow configuring format
    let mut data = vec![0; decoder.total_bytes() as usize];
    decoder.read_image(&mut data)?;

    // get original length
    let original_len = data.read_u64(data.len() - U64_BYTES);

    // write file to out
    let mut out = io::BufWriter::new(out);
    out.write_all(&data[0..original_len as usize])?;
    out.flush()?;

    Ok(())
}