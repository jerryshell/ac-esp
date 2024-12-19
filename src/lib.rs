mod model;
mod offset;
mod util;

use anyhow::Result;
use std::{
    sync::{Arc, RwLock},
    thread,
    time::{Duration, Instant},
};
use windows::{
    core::s,
    Win32::{
        Foundation::{HINSTANCE, RECT},
        System::{LibraryLoader::GetModuleHandleA, SystemServices::DLL_PROCESS_ATTACH},
        UI::WindowsAndMessaging::{FindWindowA, GetWindowInfo, WINDOWINFO},
    },
};

const FRAME_RATE: u64 = 60;

fn run() -> Result<()> {
    let draw_rect_list = Arc::new(RwLock::new(Vec::<RECT>::with_capacity(32)));

    let game_window = unsafe { FindWindowA(None, s!("AssaultCube")) }?;

    let mut window_info = WINDOWINFO::default();
    unsafe { GetWindowInfo(game_window, &mut window_info) }?;

    let draw_rect_list_clone = Arc::clone(&draw_rect_list);
    thread::spawn(move || {
        let mut overlay = windows_ez_overlay::Overlay::new(
            window_info.rcClient.left,
            window_info.rcClient.top,
            window_info.rcClient.right,
            window_info.rcClient.bottom,
            draw_rect_list_clone,
            FRAME_RATE,
            true,
        );
        overlay.pen_width = 2;
        let _ = overlay.window_loop();
    });

    let window_width = window_info.rcClient.right - window_info.rcClient.left;
    let window_height = window_info.rcClient.bottom - window_info.rcClient.top;

    let module_base_addr = unsafe { GetModuleHandleA(s!("ac_client.exe")).map(|h| h.0 as u32) }?;

    let entity_list_base_addr = util::read_memory::<u32>(module_base_addr, offset::ENTITY_LIST);

    read_game_data_loop(
        module_base_addr,
        entity_list_base_addr,
        window_width,
        window_height,
        draw_rect_list,
    );

    Ok(())
}

fn read_game_data_loop(
    module_base_addr: u32,
    entity_list_base_addr: u32,
    window_width: i32,
    window_height: i32,
    draw_rect_list: Arc<RwLock<Vec<RECT>>>,
) {
    let tick_rate = Duration::from_millis(1000 / FRAME_RATE);
    let mut last_tick = Instant::now();
    loop {
        let player_count = util::read_memory::<u32>(module_base_addr, offset::PLAYER_COUNT);
        let view_matrix = util::read_memory::<[f32; 16]>(module_base_addr, offset::VIEW_MATRIX);

        let new_draw_rect_list = (1..player_count)
            .filter_map(|i| {
                let entity_offset = i * 0x4;
                let entity_base_addr =
                    util::read_memory::<u32>(entity_list_base_addr, entity_offset);
                let entity = model::Entity {
                    base_addr: entity_base_addr,
                };

                if entity.health() <= 0 {
                    return None;
                }

                let head_screen_position = match util::world_to_screen(
                    entity.head_position(),
                    view_matrix,
                    window_width,
                    window_height,
                ) {
                    Some(screen_position) => screen_position,
                    None => return None,
                };

                let feet_screen_position = match util::world_to_screen(
                    entity.feet_position(),
                    view_matrix,
                    window_width,
                    window_height,
                ) {
                    Some(screen_position) => screen_position,
                    None => return None,
                };

                let rect_height = (feet_screen_position.y - head_screen_position.y) as i32;
                let rect_width = rect_height / 2;
                let rect_left = head_screen_position.x as i32 - rect_width / 2;
                let rect_top = head_screen_position.y as i32;
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

        if let Ok(mut draw_rect_list) = draw_rect_list.write() {
            draw_rect_list.clear();
            draw_rect_list.extend(new_draw_rect_list);
        }

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        thread::sleep(timeout);
        last_tick = Instant::now();
    }
}

#[no_mangle]
extern "system" fn DllMain(_dll_module: HINSTANCE, call_reason: u32, _reserved: *mut ()) -> bool {
    if call_reason == DLL_PROCESS_ATTACH {
        thread::spawn(move || {
            let _ = run();
        });
    }
    true
}
