pub(crate) mod chunk;

use chunk::Chunk;

const PNG_SIGNATURE: &[u8] = &[137, 80, 78, 71, 13, 10, 26, 10];

fn big_endian_to_uint32(chunk: &[u8]) -> u32 {
    u32::from(chunk[3])
        | u32::from(chunk[2]) << 8
        | u32::from(chunk[1]) << 16
        | u32::from(chunk[0]) << 24
}

#[derive(Debug, PartialEq, Eq)]
pub enum DecoderError {
    IsNotPngImage,
    IHDRWrongSize,
    UnkownTagFound,
}

#[derive(Clone, Debug)]
pub struct Decoder {
    start_offset: usize,

    buffer: Vec<u8>,
    chunks: Vec<Chunk>,
}

impl Decoder {
    pub fn new(buffer: Vec<u8>) -> Self {
        Self {
            start_offset: 0,
            chunks: Vec::new(),
            buffer,
        }
    }

    fn ignore_chunk(&mut self, length: u32) {
        self.start_offset += length as usize; // chunk length
        self.start_offset += 4; // checksum
    }

    fn parse_ihdr(&mut self, length: u32) -> Result<(), DecoderError> {
        if length != 13 {
            return Err(DecoderError::IHDRWrongSize);
        }

        let raw = &self.buffer[self.start_offset..self.start_offset + length as usize];

        let raw_width = &self.buffer[self.start_offset..self.start_offset + 4];
        let width = big_endian_to_uint32(raw_width);
        self.start_offset += 4;

        let raw_height = &self.buffer[self.start_offset..self.start_offset + 4];
        let height = big_endian_to_uint32(raw_height);
        self.start_offset += 4;

        let bit_depth = &self.buffer[self.start_offset..self.start_offset + 1];
        self.start_offset += 1;

        let colour_type = &self.buffer[self.start_offset..self.start_offset + 1];
        self.start_offset += 1;

        let compression_method = &self.buffer[self.start_offset..self.start_offset + 1];
        self.start_offset += 1;

        let filter_method = &self.buffer[self.start_offset..self.start_offset + 1];
        self.start_offset += 1;

        let interlace_method = &self.buffer[self.start_offset..self.start_offset + 1];
        self.start_offset += 1;

        self.chunks.push(Chunk::Ihdr {
            bit_depth: bit_depth[0],
            colour_type: colour_type[0],
            compression_method: compression_method[0],
            filter_method: filter_method[0],
            interlace_method: interlace_method[0],
            raw: raw.to_vec(),
            width,
            height,
        });

        self.start_offset += 4;

        Ok(())
    }

    // bad with names
    fn get_raw_len(&self, raw: &[u8]) -> usize {
        let index = raw.iter().position(|&b| b == 0).unwrap();
        match index {
            0 => 1,
            _ => index + 1,
        }
    }

    fn parse_text(&mut self, length: u32) -> Result<(), DecoderError> {
        let raw = &self.buffer[self.start_offset..self.start_offset + length as usize];

        let keyword_len = self.get_raw_len(raw);
        let keyword = &self.buffer[self.start_offset..self.start_offset + keyword_len - 1];
        let keyword = String::from_utf8_lossy(keyword).to_string();
        self.start_offset += keyword_len;

        let delta = length as usize - keyword_len;
        let text_string = &self.buffer[self.start_offset..self.start_offset + delta];
        let text_string = String::from_utf8_lossy(text_string).to_string();
        self.start_offset += delta;
        self.start_offset += 4;

        self.chunks.push(Chunk::Text {
            keyword,
            text_string,
            raw: raw.to_vec(),
        });

        Ok(())
    }

    fn parse_itxt(&mut self, length: u32) -> Result<(), DecoderError> {
        let raw = &self.buffer[self.start_offset..self.start_offset + length as usize];

        let mut keyword_len = self.get_raw_len(raw);
        let keyword = &self.buffer[self.start_offset..self.start_offset + keyword_len - 1];
        let keyword = String::from_utf8_lossy(keyword).to_string();
        self.start_offset += keyword_len;

        let compression_flag = &self.buffer[self.start_offset..self.start_offset + 1];
        self.start_offset += 1;
        keyword_len += 1;

        let compression_method = &self.buffer[self.start_offset..self.start_offset + 1];
        self.start_offset += 1;
        keyword_len += 1;

        let language_tag_len = self.get_raw_len(&raw[keyword_len..]);
        let language_tag =
            &self.buffer[self.start_offset..self.start_offset + language_tag_len - 1];
        let language_tag = String::from_utf8_lossy(language_tag).to_string();
        self.start_offset += language_tag_len;

        let translated_keyword_len = self.get_raw_len(&raw[keyword_len + language_tag_len..]);
        let translated_keyword =
            &self.buffer[self.start_offset..self.start_offset + translated_keyword_len - 1];
        let translated_keyword = String::from_utf8_lossy(translated_keyword).to_string();
        self.start_offset += translated_keyword_len;

        let delta = length as usize - (keyword_len + language_tag_len + translated_keyword_len);
        let text = &self.buffer[self.start_offset..self.start_offset + delta];
        let text = String::from_utf8_lossy(text).to_string();
        self.start_offset += delta;
        self.start_offset += 4;

        self.chunks.push(Chunk::Itxt {
            keyword,
            compression_flag: compression_flag[0],
            compression_method: compression_method[0],
            language_tag,
            translated_keyword,
            text,
            raw: raw.to_vec(),
        });

        Ok(())
    }

    fn parse_chunks(&mut self) -> Result<bool, DecoderError> {
        let length = &self.buffer[self.start_offset..self.start_offset + 4];
        let length = big_endian_to_uint32(length);
        self.start_offset += 4;

        let chunk_type = &self.buffer[self.start_offset..self.start_offset + 4];
        let chunk_type = String::from_utf8_lossy(chunk_type).to_string();
        self.start_offset += 4;

        match chunk_type.as_str() {
            "IHDR" => {
                self.parse_ihdr(length)?;
                Ok(false)
            }
            "tEXt" => {
                self.parse_text(length)?;
                Ok(false)
            }
            "iTXt" => {
                self.parse_itxt(length)?;
                Ok(false)
            }
            "IEND" => Ok(true),
            _ => {
                self.ignore_chunk(length);
                Ok(false)
            }
        }
    }

    pub fn decode(&mut self) -> Result<Vec<Chunk>, DecoderError> {
        let signature = &self.buffer[self.start_offset..self.start_offset + 8];
        self.start_offset += 8;

        match signature {
            PNG_SIGNATURE => {
                while !self.parse_chunks()? {}
                Ok(self.chunks.clone())
            }
            _ => Err(DecoderError::IsNotPngImage),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Chunk, Decoder};

    use std::{
        fs::{self, File},
        io::{self, Read},
        path::Path,
    };

    fn read_file(path: &str) -> Result<Vec<u8>, io::Error> {
        let path = Path::new(path);
        let mut file = File::open(path)?;
        let metadata = fs::metadata(path)?;
        let mut buffer = vec![0; metadata.len() as usize];
        _ = file.read(&mut buffer)?;
        Ok(buffer)
    }

    #[test]
    fn it_should_be_ok() {
        let buffer = read_file("./src/images/test_image.png").expect("err");
        let mut decoder = Decoder::new(buffer);
        let metadata = decoder.decode().unwrap();

        assert_eq!(metadata.len(), 3);

        for parsed_chunk in metadata.iter() {
            match parsed_chunk {
                Chunk::Ihdr {
                    width,
                    height,
                    bit_depth,
                    colour_type,
                    compression_method,
                    filter_method,
                    interlace_method,
                    ..
                } => {
                    assert_eq!(*width, 800);
                    assert_eq!(*height, 1113);
                    assert_eq!(*bit_depth, 8);
                    assert_eq!(*colour_type, 6);
                    assert_eq!(*compression_method, 0);
                    assert_eq!(*filter_method, 0);
                    assert_eq!(*interlace_method, 0);
                }
                Chunk::Text {
                    keyword,
                    text_string,
                    ..
                } => {
                    assert_eq!(*keyword, "Software".to_string());
                    assert_eq!(*text_string, "Adobe ImageReady".to_string());
                }
                Chunk::Itxt {
                    keyword,
                    compression_flag,
                    compression_method,
                    language_tag,
                    translated_keyword,
                    ..
                } => {
                    assert_eq!(*keyword, "XML:com.adobe.xmp".to_string());
                    assert_eq!(*compression_flag, 0);
                    assert_eq!(*compression_method, 0);
                    assert_eq!((*language_tag).len(), 0);
                    assert_eq!((*translated_keyword).len(), 0);
                }
            }
        }
    }

    #[test]
    fn it_should_be_ok_with_test_image2() {
        let buffer = read_file("./src/images/test_image2.png").expect("err");
        let mut decoder = Decoder::new(buffer);
        let metadata = decoder.decode().unwrap();

        assert_eq!(metadata.len(), 4);

        for (i, parsed_chunk) in metadata.iter().enumerate() {
            match parsed_chunk {
                Chunk::Ihdr {
                    width,
                    height,
                    bit_depth,
                    colour_type,
                    compression_method,
                    filter_method,
                    interlace_method,
                    ..
                } => {
                    assert_eq!(*width, 1440);
                    assert_eq!(*height, 1800);
                    assert_eq!(*bit_depth, 8);
                    assert_eq!(*colour_type, 2);
                    assert_eq!(*compression_method, 0);
                    assert_eq!(*filter_method, 0);
                    assert_eq!(*interlace_method, 0);
                }
                Chunk::Text {
                    keyword,
                    text_string,
                    ..
                } => match i {
                    1 => {
                        assert_eq!(*keyword, "date:create".to_string());
                        assert_eq!(*text_string, "2023-04-23T12:32:40+00:00".to_string());
                    }
                    2 => {
                        assert_eq!(*keyword, "date:modify".to_string());
                        assert_eq!(*text_string, "2023-04-23T12:32:40+00:00".to_string());
                    }
                    3 => {
                        assert_eq!(*keyword, "date:timestamp".to_string());
                        assert_eq!(*text_string, "2023-04-23T12:32:53+00:00".to_string());
                    }
                    _ => {
                        println!("This should never happen");
                        assert!(1 == 0);
                    }
                },
                _ => {
                    println!("This should never happen");
                    assert!(1 == 0);
                }
            }
        }
    }

    #[test]
    fn it_should_be_ok_with_test_image3() {
        let buffer = read_file("./src/images/test_image3.png").expect("err");
        let mut decoder = Decoder::new(buffer);
        let metadata = decoder.decode().unwrap();

        assert_eq!(metadata.len(), 3);

        for (i, parsed_chunk) in metadata.iter().enumerate() {
            match parsed_chunk {
                Chunk::Ihdr {
                    width,
                    height,
                    bit_depth,
                    colour_type,
                    compression_method,
                    filter_method,
                    interlace_method,
                    ..
                } => {
                    assert_eq!(*width, 1245);
                    assert_eq!(*height, 789);
                    assert_eq!(*bit_depth, 8);
                    assert_eq!(*colour_type, 6);
                    assert_eq!(*compression_method, 0);
                    assert_eq!(*filter_method, 0);
                    assert_eq!(*interlace_method, 0);
                }
                Chunk::Text {
                    keyword,
                    text_string,
                    ..
                } => match i {
                    1 => {
                        assert_eq!(*keyword, "Software".to_string());
                        assert_eq!(*text_string, "gnome-screenshot".to_string());
                    }
                    2 => {
                        assert_eq!(*keyword, "Creation Time".to_string());
                        assert_eq!(*text_string, "Tue 17 Jan 2023 10:44:41 AM CET".to_string());
                    }
                    _ => {
                        println!("This should never happen");
                        assert!(1 == 0);
                    }
                },
                _ => {
                    // this should never happen
                }
            }
        }
    }
}
