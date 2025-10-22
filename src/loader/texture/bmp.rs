use std::io::{Read, Seek};

pub struct BMP {
    pub width: i32,
    pub height: i32,
    pub pixel_data: Vec<[u8; 3]>,
}

impl BMP {
    pub fn load(path: &str) -> Self {
        let mut file = std::fs::File::open(path).unwrap();

        let identifier_buf: &mut [u8; 2] = &mut [0; 2];
        file.read_exact(identifier_buf).unwrap();
        if identifier_buf != b"BM" {
            panic!("File identifier is incorrect!");
        }

        file.seek_relative(8).unwrap();

        let image_data_offset_buf: &mut [u8; 4] = &mut [0; 4];
        file.read_exact(image_data_offset_buf).unwrap();
        let image_data_offset: i32 = i32::from_le_bytes(*image_data_offset_buf);

        file.seek_relative(4).unwrap();

        let width_buf: &mut [u8; 4] = &mut [0; 4];
        file.read_exact(width_buf).unwrap();
        let width: i32 = i32::from_le_bytes(*width_buf);

        let height_buf: &mut [u8; 4] = &mut [0; 4];
        file.read_exact(height_buf).unwrap();
        let height: i32 = i32::from_le_bytes(*height_buf);

        file.seek_relative(4).unwrap();

        let compression_method_buf: &mut [u8; 4] = &mut [0; 4];
        file.read_exact(compression_method_buf).unwrap();
        let compression_method: u32 = u32::from_le_bytes(*compression_method_buf);
        if compression_method != 0 {
            panic!("Compression method must be zero!");
        }

        let image_size_buf: &mut [u8; 4] = &mut [0; 4];
        file.read_exact(image_size_buf).unwrap();
        let image_size: u32 = u32::from_le_bytes(*image_size_buf);

        let mut pixel_byte_buffer: Vec<u8> = Vec::new();
        pixel_byte_buffer.resize(image_size as usize, 0);
        file.seek(std::io::SeekFrom::Start(image_data_offset as u64))
            .unwrap();
        file.read_exact(pixel_byte_buffer.as_mut_slice()).unwrap();

        let mut color_buffer: Vec<[u8; 3]> = Vec::new();
        pixel_byte_buffer
            .iter()
            .step_by(3)
            .enumerate()
            .for_each(|val| {
                let color: [u8; 3] = [
                    pixel_byte_buffer[(val.0 * 3) + 2],
                    pixel_byte_buffer[(val.0 * 3) + 1],
                    pixel_byte_buffer[(val.0 * 3) + 0],
                ];
                color_buffer.push(color);
            });

        return BMP {
            width,
            height,
            pixel_data: color_buffer,
        };
    }
}
