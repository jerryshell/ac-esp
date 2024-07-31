mod model;
mod offset;
mod util;

use anyhow::Result;
use std::sync::{Arc, RwLock};
use windows::{
    core::*,
    Win32::{
        Foundation::*, System::LibraryLoader::*, System::SystemServices::*,
        UI::WindowsAndMessaging::*,
    },
};

fn run() -> Result<()> {
    let refresh_interval_ms = 1000 / 60;

    let draw_rect_list = Arc::new(RwLock::new(Vec::<RECT>::with_capacity(32)));

    let game_window = unsafe { FindWindowA(None, s!("AssaultCube")) }?;

    let mut window_info = WINDOWINFO::default();
    unsafe { GetWindowInfo(game_window, &mut window_info) }?;

    let draw_rect_list_clone = Arc::clone(&draw_rect_list);
    std::thread::spawn(move || {
        let mut overlay = windows_ez_overlay::Overlay::new(
            window_info.rcClient.left,
            window_info.rcClient.top,
            window_info.rcClient.right,
            window_info.rcClient.bottom,
            draw_rect_list_clone,
            refresh_interval_ms,
            true,
        );
        overlay.pen_width = 2;
        let _ = overlay.window_loop();
    });

    let window_width = window_info.rcClient.right - window_info.rcClient.left;
    let window_height = window_info.rcClient.bottom - window_info.rcClient.top;

    let module_base_addr = unsafe { GetModuleHandleA(s!("ac_client.exe")).map(|h| h.0 as u32) }?;

    let entity_list_base_ptr = util::build_ptr(module_base_addr, offset::ENTITY_LIST);

    read_game_data_loop(
        module_base_addr,
        entity_list_base_ptr,
        window_width,
        window_height,
        draw_rect_list,
        refresh_interval_ms,
    );

    Ok(())
}

fn read_game_data_loop(
    module_base_addr: u32,
    entity_list_base_ptr: *const u32,
    window_width: i32,
    window_height: i32,
    draw_rect_list: Arc<RwLock<Vec<RECT>>>,
    refresh_interval_ms: u64,
) {
    loop {
        let start = std::time::Instant::now();

        let player_count = util::read_player_count(module_base_addr);
        let view_matrix = util::read_view_matrix(module_base_addr);

        let new_draw_rect_list = (1..player_count)
            .filter_map(|i| {
                let entity_base_ptr = util::build_entity_base_ptr(entity_list_base_ptr, i * 0x4);
                let entity = model::Entity::new(entity_base_ptr);

                if entity.health() <= 0 {
                    return None;
                }

                let mut head_screen_pos = model::Vec2::default();
                let success = util::world_to_screen(
                    entity.head_position(),
                    &mut head_screen_pos,
                    view_matrix,
                    window_width,
                    window_height,
                );
                if !success {
                    return None;
                }

                let mut feet_screen_pos = model::Vec2::default();
                let success = util::world_to_screen(
                    entity.feet_position(),
                    &mut feet_screen_pos,
                    view_matrix,
                    window_width,
                    window_height,
                );
                if !success {
                    return None;
                }

                let rect_height = (feet_screen_pos.y - head_screen_pos.y) as i32;
                let rect_width = rect_height / 2;
                let rect_left = head_screen_pos.x as i32 - rect_width / 2;
                let rect_top = head_screen_pos.y as i32;
                let rect_right = rect_left + rect_width;
                let rect_bottom = rect_top + rect_height;
                let rect = RECT {
                    left: rect_left,
                    right: rect_right,
                    top: rect_top,
                    bottom: rect_bottom,
                };

                Some(rect)
            })
            .collect::<Vec<RECT>>();

        {
            let mut draw_rect_list = draw_rect_list.write().unwrap();
            draw_rect_list.clear();
            draw_rect_list.extend(new_draw_rect_list);
        }

        let delta = start.elapsed();
        let delta_ms = delta.as_millis() as u64;
        if refresh_interval_ms > delta_ms {
            std::thread::sleep(std::time::Duration::from_millis(
                refresh_interval_ms - delta_ms,
            ));
        }
    }
}

#[no_mangle]
extern "system" fn DllMain(_dll_module: HINSTANCE, call_reason: u32, _reserved: *mut ()) -> bool {
    if call_reason == DLL_PROCESS_ATTACH {
        std::thread::spawn(move || {
            let _ = run();
        });
    }
    true
}
