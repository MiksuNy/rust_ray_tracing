use crate::loader::texture::bmp::BMP;

mod bmp;

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
