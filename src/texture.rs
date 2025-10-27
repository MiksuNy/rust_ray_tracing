use crate::loader::bmp::BMP;

#[derive(Clone, Default)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub pixel_data: Vec<[u8; 3]>,
}

impl Texture {
    pub fn load_from_bmp(path: &str) -> Self {
        return BMP::load(path).into();
    }

    pub fn color_at(&self, uv: [f32; 2]) -> [u8; 3] {
        let i: usize = (uv[0] * self.width as f32) as usize;
        let j: usize = (uv[1] * self.height as f32) as usize;
        let mut index: usize = i + (j * self.width);
        while index > self.pixel_data.len() - 1 {
            index -= self.pixel_data.len() - 1;
        }
        return self.pixel_data[index];
    }
}

impl From<BMP> for Texture {
    fn from(bmp: BMP) -> Self {
        return Self {
            width: bmp.width as usize,
            height: bmp.height as usize,
            pixel_data: bmp.pixel_data,
        };
    }
}
