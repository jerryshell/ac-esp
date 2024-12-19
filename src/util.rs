use crate::model;

pub fn world_to_screen(
    world_position: model::Vec3,
    view_matrix: [f32; 16],
    window_width: i32,
    window_height: i32,
) -> Option<model::Vec2> {
    let w = world_position.x * view_matrix[3]
        + world_position.y * view_matrix[7]
        + world_position.z * view_matrix[11]
        + view_matrix[15];

    if w < 0.001 {
        return None;
    }

    let x = world_position.x * view_matrix[0]
        + world_position.y * view_matrix[4]
        + world_position.z * view_matrix[8]
        + view_matrix[12];
    let y = world_position.x * view_matrix[1]
        + world_position.y * view_matrix[5]
        + world_position.z * view_matrix[9]
        + view_matrix[13];

    let nx = x / w;
    let ny = y / w;

    let window_center_x = (window_width / 2) as f32;
    let window_center_y = (window_height / 2) as f32;

    let screen_position = model::Vec2 {
        x: window_center_x + (window_center_x * nx),
        y: window_center_y - (window_center_y * ny),
    };

    Some(screen_position)
}

pub fn read_memory<T>(base_addr: u32, offset: u32) -> T
where
    T: Copy,
{
    let data_ptr = (base_addr + offset) as *const T;
    unsafe { *data_ptr }
}
