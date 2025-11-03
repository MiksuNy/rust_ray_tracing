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
        let i: i32 = (uv[0] * self.width as f32) as i32;
        let j: i32 = (uv[1] * self.height as f32) as i32;
        let mut index: i32 = i + (j * self.width as i32);
        while index > self.pixel_data.len() as i32 - 1 {
            index -= self.pixel_data.len() as i32 - 1;
        }
        while index < 0 {
            index += self.pixel_data.len() as i32 - 1;
        }
        return self.pixel_data[index as usize];
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
