use pngmet::{Decoder, DecoderError};
use std::env;
use std::{
    fs::{self, File},
    io::{self, Read},
    path::Path,
};

const HELP_TEXT: &str = "Usage:\n    ./pngmet [image_path]";

fn read_file(path: &str) -> Result<Vec<u8>, io::Error> {
    let path = Path::new(path);
    let mut file = File::open(path)?;
    let metadata = fs::metadata(path)?;
    let mut buffer = vec![0; metadata.len() as usize];
    _ = file.read(&mut buffer)?;
    Ok(buffer)
}

fn main() -> Result<(), DecoderError> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => println!("{HELP_TEXT}"),
        _ => {
            let image_path = &args[1];
            let image_buff = read_file(image_path).expect("Error reading the image.");
            let mut decoder = Decoder::new(image_buff);
            let chunks = decoder.decode()?;
            for chunk in chunks.iter() {
                println!("{chunk}");
            }
        }
    };

    Ok(())
}
