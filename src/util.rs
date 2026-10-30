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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Vec3;

    const IDENTITY: [f32; 16] = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];

    #[test]
    fn viewport_center_projects_to_screen_center() {
        let screen = world_to_screen(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            IDENTITY,
            800,
            600,
        )
        .unwrap();

        assert_eq!((screen.x, screen.y), (400.0, 300.0));
    }

    #[test]
    fn world_right_and_up_project_right_and_up() {
        let screen = world_to_screen(
            Vec3 {
                x: 0.5,
                y: 0.5,
                z: 0.0,
            },
            IDENTITY,
            800,
            600,
        )
        .unwrap();

        assert_eq!((screen.x, screen.y), (600.0, 150.0));
    }

    #[test]
    fn projection_divides_by_w() {
        let mut matrix = IDENTITY;
        matrix[15] = 2.0;

        let screen = world_to_screen(
            Vec3 {
                x: 1.0,
                y: -1.0,
                z: 0.0,
            },
            matrix,
            800,
            600,
        )
        .unwrap();

        assert_eq!((screen.x, screen.y), (600.0, 450.0));
    }

    #[test]
    fn projection_reads_matrix_columns_in_order() {
        let matrix = [
            0.0, 2.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];

        let screen = world_to_screen(
            Vec3 {
                x: 1.0,
                y: 2.0,
                z: 5.0,
            },
            matrix,
            800,
            600,
        )
        .unwrap();

        assert_eq!((screen.x, screen.y), (2800.0, -300.0));
    }

    #[test]
    fn point_behind_camera_has_no_screen_position() {
        let screen = world_to_screen(
            Vec3 {
                x: 1.0,
                y: 1.0,
                z: 0.0,
            },
            [0.0; 16],
            800,
            600,
        );

        assert!(screen.is_none());
    }
}
