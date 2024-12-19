use crate::{offset, util};

pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct Entity {
    pub base_addr: u32,
}

impl Entity {
    pub fn new(base_addr: u32) -> Self {
        Self { base_addr }
    }

    pub fn health(&self) -> i32 {
        util::read_memory::<i32>(self.base_addr, offset::HEALTH)
    }

    pub fn head_position(&self) -> Vec3 {
        Vec3 {
            x: util::read_memory::<f32>(self.base_addr, offset::HEAD_POSITION_X),
            y: util::read_memory::<f32>(self.base_addr, offset::HEAD_POSITION_Y),
            z: util::read_memory::<f32>(self.base_addr, offset::HEAD_POSITION_Z),
        }
    }

    pub fn feet_position(&self) -> Vec3 {
        Vec3 {
            x: util::read_memory::<f32>(self.base_addr, offset::FEET_POSITION_X),
            y: util::read_memory::<f32>(self.base_addr, offset::FEET_POSITION_Y),
            z: util::read_memory::<f32>(self.base_addr, offset::FEET_POSITION_Z),
        }
    }
}
