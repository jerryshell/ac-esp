use windows::Win32::Foundation::RECT;

#[derive(Clone, Copy)]
pub struct Viewport {
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<[f32; 3]> for Vec3 {
    fn from(xyz: [f32; 3]) -> Self {
        Vec3 {
            x: xyz[0],
            y: xyz[1],
            z: xyz[2],
        }
    }
}

struct Vec2 {
    x: f32,
    y: f32,
}

pub struct Entity {
    pub health: i32,
    pub head: Vec3,
    pub feet: Vec3,
}

pub fn screen_box(entity: &Entity, view_matrix: [f32; 16], viewport: Viewport) -> Option<RECT> {
    if entity.health <= 0 {
        return None;
    }

    let head = project(entity.head, view_matrix, viewport)?;
    let feet = project(entity.feet, view_matrix, viewport)?;

    let height = (feet.y - head.y) as i32;
    let width = height / 2;
    let left = head.x as i32 - width / 2;
    let top = head.y as i32;

    Some(RECT {
        left,
        top,
        right: left + width,
        bottom: top + height,
    })
}

fn project(position: Vec3, view_matrix: [f32; 16], viewport: Viewport) -> Option<Vec2> {
    let w = position.x * view_matrix[3]
        + position.y * view_matrix[7]
        + position.z * view_matrix[11]
        + view_matrix[15];

    if w < 0.001 {
        return None;
    }

    let x = position.x * view_matrix[0]
        + position.y * view_matrix[4]
        + position.z * view_matrix[8]
        + view_matrix[12];
    let y = position.x * view_matrix[1]
        + position.y * view_matrix[5]
        + position.z * view_matrix[9]
        + view_matrix[13];

    let center_x = (viewport.width / 2) as f32;
    let center_y = (viewport.height / 2) as f32;

    Some(Vec2 {
        x: center_x + center_x * (x / w),
        y: center_y - center_y * (y / w),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDENTITY: [f32; 16] = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    const VIEWPORT: Viewport = Viewport {
        width: 800,
        height: 600,
    };

    fn entity(health: i32, head: [f32; 3], feet: [f32; 3]) -> Entity {
        Entity {
            health,
            head: head.into(),
            feet: feet.into(),
        }
    }

    #[test]
    fn viewport_center_projects_to_screen_center() {
        let screen = project(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            IDENTITY,
            VIEWPORT,
        )
        .unwrap();

        assert_eq!((screen.x, screen.y), (400.0, 300.0));
    }

    #[test]
    fn world_right_and_up_project_right_and_up() {
        let screen = project(
            Vec3 {
                x: 0.5,
                y: 0.5,
                z: 0.0,
            },
            IDENTITY,
            VIEWPORT,
        )
        .unwrap();

        assert_eq!((screen.x, screen.y), (600.0, 150.0));
    }

    #[test]
    fn projection_divides_by_w() {
        let mut matrix = IDENTITY;
        matrix[15] = 2.0;

        let screen = project(
            Vec3 {
                x: 1.0,
                y: -1.0,
                z: 0.0,
            },
            matrix,
            VIEWPORT,
        )
        .unwrap();

        assert_eq!((screen.x, screen.y), (600.0, 450.0));
    }

    #[test]
    fn projection_reads_matrix_columns_in_order() {
        let matrix = [
            0.0, 2.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];

        let screen = project(
            Vec3 {
                x: 1.0,
                y: 2.0,
                z: 5.0,
            },
            matrix,
            VIEWPORT,
        )
        .unwrap();

        assert_eq!((screen.x, screen.y), (2800.0, -300.0));
    }

    #[test]
    fn point_behind_camera_has_no_screen_position() {
        let screen = project(
            Vec3 {
                x: 1.0,
                y: 1.0,
                z: 0.0,
            },
            [0.0; 16],
            VIEWPORT,
        );

        assert!(screen.is_none());
    }

    #[test]
    fn living_entity_becomes_head_to_feet_box() {
        let entity = entity(100, [0.5, 0.5, 0.0], [0.5, -0.5, 0.0]);

        let rect = screen_box(&entity, IDENTITY, VIEWPORT).unwrap();

        assert_eq!(
            (rect.left, rect.top, rect.right, rect.bottom),
            (525, 150, 675, 450)
        );
    }

    #[test]
    fn dead_entity_has_no_box() {
        let entity = entity(0, [0.5, 0.5, 0.0], [0.5, -0.5, 0.0]);

        assert!(screen_box(&entity, IDENTITY, VIEWPORT).is_none());
    }

    #[test]
    fn entity_behind_camera_has_no_box() {
        let entity = entity(100, [0.5, 0.5, 0.0], [0.5, -0.5, 0.0]);

        assert!(screen_box(&entity, [0.0; 16], VIEWPORT).is_none());
    }
}
