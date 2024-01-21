use crate::{offset, util};

#[derive(Default)]
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
    pub base_ptr: *const u32,
}

impl Entity {
    pub fn new(base_ptr: *const u32) -> Entity {
        Entity { base_ptr }
    }

    pub fn health(&self) -> i32 {
        util::read_memory::<i32>(self.base_ptr, offset::HEALTH)
    }

    pub fn head_position(&self) -> Vec3 {
        Vec3 {
            x: util::read_memory::<f32>(self.base_ptr, offset::HEAD_POSITION_X),
            y: util::read_memory::<f32>(self.base_ptr, offset::HEAD_POSITION_Y),
            z: util::read_memory::<f32>(self.base_ptr, offset::HEAD_POSITION_Z),
        }
    }

    pub fn feet_position(&self) -> Vec3 {
        Vec3 {
            x: util::read_memory::<f32>(self.base_ptr, offset::FEET_POSITION_X),
            y: util::read_memory::<f32>(self.base_ptr, offset::FEET_POSITION_Y),
            z: util::read_memory::<f32>(self.base_ptr, offset::FEET_POSITION_Z),
        }
    }
}
