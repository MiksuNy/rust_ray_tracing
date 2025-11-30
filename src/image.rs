use crate::{log_error, log_info};
use std::io::Write;

#[derive(Default)]
pub struct Image {
    pub format: ImageFormat,
    pub width: usize,
    pub height: usize,
    pub bytes: Vec<u8>,
}

#[derive(Default)]
pub enum ImageFormat {
    #[default]
    PPM,
}

impl Image {
    pub fn new(format: ImageFormat, width: usize, height: usize) -> Self {
        return Self {
            format,
            width,
            height,
            bytes: Vec::new(),
        };
    }

    pub fn write_to_path(&self, path: &str) {
        match self.format {
            ImageFormat::PPM => {
                let mut output_file = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(path)
                    .unwrap();
                output_file
                    .write_fmt(format_args!("P3\n{} {}\n255\n", self.width, self.height))
                    .unwrap();

                let mut buffer: Vec<u8> = Vec::new();
                buffer.reserve_exact(self.bytes.len() * 2);
                (0..self.bytes.len()).step_by(3).for_each(|index: usize| {
                    let _ = buffer.write_fmt(format_args!(
                        "{} {} {} ",
                        self.bytes[index + 0],
                        self.bytes[index + 1],
                        self.bytes[index + 2]
                    ));
                    if index % self.width == 0 && index != 0 {
                        let _ = buffer.write(b"\n");
                    }
                });
                let result = output_file.write(buffer.as_slice());

                if result.is_ok() {
                    log_info!("Image data succesfully written to '{}'", path);
                } else {
                    log_error!(
                        "Could not write image data to '{}' with error '{:?}'",
                        path,
                        result
                    );
                }
            }
        }
    }
}
