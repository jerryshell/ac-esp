use crate::{model, offset};

pub fn world_to_screen(
    position: model::Vec3,
    screen: &mut model::Vec2,
    view_matrix: [f32; 16],
    window_width: i32,
    window_height: i32,
) -> bool {
    let w = position.x * view_matrix[3]
        + position.y * view_matrix[7]
        + position.z * view_matrix[11]
        + view_matrix[15];

    if w < 0.001 {
        return false;
    }

    let x = position.x * view_matrix[0]
        + position.y * view_matrix[4]
        + position.z * view_matrix[8]
        + view_matrix[12];
    let y = position.x * view_matrix[1]
        + position.y * view_matrix[5]
        + position.z * view_matrix[9]
        + view_matrix[13];

    let nx = x / w;
    let ny = y / w;

    let window_center_x = (window_width / 2) as f32;
    let window_center_y = (window_height / 2) as f32;

    screen.x = window_center_x + (window_center_x * nx);
    screen.y = window_center_y - (window_center_y * ny);

    true
}

pub fn build_ptr(base: u32, offset: u32) -> *const u32 {
    (base + offset) as *const u32
}

pub fn build_entity_base_ptr(entity_list_base_ptr: *const u32, offset: u32) -> *const u32 {
    unsafe {
        let entity_list_base_ptr_deref = *entity_list_base_ptr;
        build_ptr(entity_list_base_ptr_deref, offset)
    }
}

pub fn read_memory<T>(base_ptr: *const u32, offset: u32) -> T
where
    T: Copy,
{
    unsafe {
        let base_ptr_deref = *base_ptr;
        let data_ptr = (base_ptr_deref + offset) as *const T;
        *data_ptr
    }
}

pub fn read_player_count(module_base_addr: u32) -> u32 {
    let player_count_ptr = build_ptr(module_base_addr, offset::PLAYER_COUNT);
    unsafe { *player_count_ptr }
}

pub fn read_view_matrix(module_base_addr: u32) -> [f32; 16] {
    let view_matrix_ptr = (module_base_addr + offset::VIEW_MATRIX) as *const [f32; 16];
    unsafe { *view_matrix_ptr }
}
