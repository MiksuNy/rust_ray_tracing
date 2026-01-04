use crate::{
    log_error,
    vector::{Vec2Swizzles, Vec2f},
};

#[derive(Clone, Default)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub pixel_data: Vec<[u8; 3]>,
}

impl Texture {
    pub fn load(path: &str) -> Option<Self> {
        if !std::fs::exists(path).unwrap() {
            log_error!("Could not find texture at path: '{}'", path);
            return None;
        }

        let img = image::open(path).unwrap().flipv().to_rgb8();

        Some(Self {
            width: img.width() as usize,
            height: img.height() as usize,
            pixel_data: img.pixels().map(|pixel| pixel.0).collect(),
        })
    }

    pub fn color_at(&self, uv: Vec2f) -> [u8; 3] {
        let i: i32 = (uv.x() * self.width as f32) as i32;
        let j: i32 = (uv.y() * self.height as f32) as i32;
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
