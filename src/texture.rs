use crate::{
    log_error,
    vector::{Vec2Swizzles, Vec2f},
};

#[derive(Clone, Default)]
pub struct Texture {
    pub hash: u32,
    pub width: usize,
    pub height: usize,
    pub pixel_data: Vec<[u8; 4]>,
}

impl Texture {
    pub fn load(path: &str) -> Option<Self> {
        if !std::fs::exists(path).unwrap() {
            log_error!("Could not find texture at path: '{}'", path);
            return None;
        }

        let img = image::open(path).unwrap().flipv().to_rgba8();
        let pixel_data: Vec<[u8; 4]> = img
            .pixels()
            .map(|pixel| [pixel.0[0], pixel.0[1], pixel.0[2], pixel.0[3]])
            .collect();
        let hash = Self::calculate_djb2_hash(pixel_data.as_slice());

        Some(Self {
            hash,
            width: img.width() as usize,
            height: img.height() as usize,
            pixel_data,
        })
    }

    pub fn color_at(&self, uv: Vec2f) -> [u8; 4] {
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

    fn calculate_djb2_hash(pixel_data: &[[u8; 4]]) -> u32 {
        let mut hash: u32 = 5381;
        for i in 0..pixel_data.len() {
            let color = bytemuck::from_bytes::<u32>(&pixel_data[i]);
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(*color);
        }
        return hash;
    }
}
