use std::fmt::Display;

#[derive(Clone, Debug)]
pub enum Chunk {
    Ihdr {
        width: u32,
        height: u32,
        bit_depth: u8,
        colour_type: u8,
        compression_method: u8,
        filter_method: u8,
        interlace_method: u8,
        raw: Vec<u8>,
    },
    Text {
        keyword: String,
        text_string: String,
        raw: Vec<u8>,
    },
    Itxt {
        keyword: String,
        compression_flag: u8,
        compression_method: u8,
        language_tag: String,
        translated_keyword: String,
        text: String,
        raw: Vec<u8>,
    },
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ihdr {
                width,
                height,
                bit_depth,
                colour_type,
                compression_method,
                filter_method,
                interlace_method,
                ..
            } => {
                    let colour_type = match *colour_type {
                        0 => format!("Greyscale({colour_type})"), 
                        2 => format!("RGB({colour_type})"), 
                        3 => format!("Indexed color({colour_type})"), 
                        4 => format!("Greyscale Alpha({colour_type})"), 
                        6 => format!("RGBA({colour_type})"), 
                        _ => format!("Unknown colour type({colour_type})")
                    };

                    let compression_method = if *compression_method == 0 {
                        format!("DEFLATE({compression_method})")
                    } else {
                        format!("Unknown compression method({compression_method})")
                    };

                    let filter_method = if *filter_method == 0 {
                        format!("Adaptive({filter_method})")
                    } else {
                        format!("Unknown filter method({filter_method})")
                    };

                    let interlace_method = if *interlace_method == 0 {
                        format!("No interlace ({interlace_method})")
                    } else if *interlace_method == 1 {
                        format!("Adam7({interlace_method})")
                    } else {
                        format!("Unknown interlace method({interlace_method})")
                    };


                    write!(
                        f,
                        "Width:  {width} pixels
height: {height} pixels
Bit Depth: {bit_depth} bits per channel
colour_type: {colour_type}
compression_method: {compression_method}
filter_method: {filter_method}
interlace_method: {interlace_method}",
                    )
                },
            Self::Text {
                keyword,
                text_string,
                ..
            } => write!(
                f,
                "
Keyword: {keyword}
Text String: {text_string}"
            ),
            Self::Itxt {
                keyword,
                compression_flag,
                compression_method,
                language_tag,
                translated_keyword,
                text,
                ..
            } => {
                    let compression_method = if *compression_method == 0 && *compression_flag == 1 {
                        format!("Zlib compression method({compression_method})")
                    } else {
                        format!("Uncompressed({compression_method})")
                    };

                    let compression_flag = if *compression_flag == 0 {
                        format!("Uncompressed({compression_flag})")
                    } else if *compression_flag == 1 {
                        format!("Compressed({compression_flag})")
                    } else {
                        format!("Unknown compression flag({compression_flag})")
                    };

                    write!(
                        f,
                        "
Keyword: {keyword}
Compression flag: {compression_flag}
Compression method: {compression_method}
Language Tag: {language_tag}
Translated Keyword: {translated_keyword}
Text: {text}"
                    )

                },
        }
    }
}

