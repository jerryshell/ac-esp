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
    pub fn health(&self) -> i32 {
        util::read_memory::<i32>(self.base_addr, offset::ENTITY_HEALTH)
    }

    pub fn head_position(&self) -> Vec3 {
        let xyz = util::read_memory::<[f32; 3]>(self.base_addr, offset::ENTITY_HEAD_POSITION);
        Vec3 {
            x: xyz[0],
            y: xyz[1],
            z: xyz[2],
        }
    }

    pub fn feet_position(&self) -> Vec3 {
        let xyz = util::read_memory::<[f32; 3]>(self.base_addr, offset::ENTITY_FEET_POSITION);
        Vec3 {
            x: xyz[0],
            y: xyz[1],
            z: xyz[2],
        }
    }
}
