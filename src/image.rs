use std::io::Write;

pub trait Image {
    fn new(width: usize, height: usize) -> Self;
    fn write_to_path(&self, path: &str);
}

pub struct PPM {
    pub width: usize,
    pub height: usize,
    pub pixel_data: Vec<[u8; 3]>,
}

impl Image for PPM {
    fn new(width: usize, height: usize) -> Self {
        return Self {
            width,
            height,
            pixel_data: Vec::new(),
        };
    }

    fn write_to_path(&self, path: &str) {
        let mut output_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .unwrap();
        output_file
            .write_fmt(format_args!("P3\n{} {}\n255\n", self.width, self.height))
            .unwrap();

        let mut buffer: Vec<u8> = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let color = self.pixel_data[x + (y * self.width)];
                let _ = buffer.write_fmt(format_args!("{} {} {} ", color[0], color[1], color[2]));
            }
            let _ = buffer.write(b"\n");
        }
        let _ = output_file.write(buffer.as_slice());
    }
}
