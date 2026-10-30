use crate::esp::Entity;

pub const PLAYER_COUNT: u32 = 0x18AC0C;
pub const VIEW_MATRIX: u32 = 0x17DFD0;
pub const ENTITY_LIST: u32 = 0x18AC04;
pub const ENTITY_HEAD_POSITION: u32 = 0x4;
pub const ENTITY_FEET_POSITION: u32 = 0x28;
pub const ENTITY_HEALTH: u32 = 0xEC;

pub fn read<T>(base_addr: u32, offset: u32) -> T
where
    T: Copy,
{
    let data_ptr = (base_addr + offset) as *const T;
    unsafe { *data_ptr }
}

pub fn read_entity(entity_list_base_addr: u32, index: u32) -> Entity {
    let base_addr = read::<u32>(entity_list_base_addr, index * 4);

    Entity {
        health: read(base_addr, ENTITY_HEALTH),
        head: read::<[f32; 3]>(base_addr, ENTITY_HEAD_POSITION).into(),
        feet: read::<[f32; 3]>(base_addr, ENTITY_FEET_POSITION).into(),
    }
}
