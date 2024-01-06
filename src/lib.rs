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
    let draw_rect_list = Arc::new(RwLock::new(Vec::<RECT>::with_capacity(32)));
    unsafe {
        let game_window = FindWindowA(None, s!("AssaultCube"));

        let mut window_info = WINDOWINFO::default();
        GetWindowInfo(game_window, &mut window_info)?;

        let draw_rect_list_clone = Arc::clone(&draw_rect_list);
        std::thread::spawn(move || {
            let mut overlay = windows_ez_overlay::Overlay::new(
                window_info.rcClient.left,
                window_info.rcClient.top,
                window_info.rcClient.right,
                window_info.rcClient.bottom,
                draw_rect_list_clone,
                1000 / 30,
                true,
            );
            let _ = overlay.window_loop();
        });

        let window_width = window_info.rcClient.right - window_info.rcClient.left;
        let window_height = window_info.rcClient.bottom - window_info.rcClient.top;

        let module_base_addr = GetModuleHandleA(s!("ac_client.exe")).map(|h| h.0 as u32)?;

        let entity_list_base_ptr = util::build_ptr(module_base_addr, offset::ENTITY_LIST);

        loop {
            let player_count = util::read_player_count(module_base_addr);
            let view_matrix = util::read_view_matrix(module_base_addr);

            let mut new_draw_rect_list = Vec::with_capacity(player_count as usize);
            for i in 1..player_count {
                let entity_base_ptr = util::build_ptr(*entity_list_base_ptr, i * 0x4);
                let entity = model::Entity::new(entity_base_ptr);

                if entity.health() <= 0 {
                    continue;
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
                    continue;
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
                    continue;
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

                new_draw_rect_list.push(rect);
            }

            {
                let mut draw_rect_list = draw_rect_list.write().unwrap();
                draw_rect_list.clear();
                draw_rect_list.extend(new_draw_rect_list);
            }

            std::thread::sleep(std::time::Duration::from_millis(1000 / 30));
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
extern "system" fn DllMain(_dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    if call_reason == DLL_PROCESS_ATTACH {
        std::thread::spawn(move || {
            let _ = run();
        });
    }
    true
}
