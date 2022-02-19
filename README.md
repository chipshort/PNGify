# PNGify

This is a small tool that allows you to put any binary data into a grayscale image where each byte is used as one pixel.  
This can be used to upload arbitrary files on an image hoster (as long as they do not compress / re-encode uploads).

The tool can either take files as input / output or stdin and stdout. It currently supports png and pnm output / input.

## Usage

```bash
pngify encode -i data_file -o image_file.png
pngify decode -i image_file.png -o decoded_data_file
```