use crate::loader::bmp::BMP;

#[derive(Default)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub pixel_data: Vec<[u8; 3]>,
}

impl Texture {
    pub fn load_bmp(path: &str) -> Self {
        return BMP::load(path).into();
    }

    pub fn color_at(&self, tex_coord: [f32; 2]) -> [u8; 3] {
        return self.pixel_data[((tex_coord[0] * self.width as f32)
            + ((tex_coord[1] * self.height as f32) * self.width as f32))
            as usize];
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
