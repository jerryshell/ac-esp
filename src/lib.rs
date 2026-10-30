mod ac_client;
mod esp;
pub mod overlay;

use anyhow::Result;
use esp::Viewport;
use std::{
    sync::{Arc, RwLock},
    thread,
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::*,
        System::{LibraryLoader::*, SystemServices::*},
        UI::WindowsAndMessaging::*,
    },
    core::*,
};

const FRAME_RATE: u64 = 60;
const TICK_RATE: Duration = Duration::from_millis(1000 / FRAME_RATE);

fn run() -> Result<()> {
    let draw_rect_list = Arc::new(RwLock::new(Vec::<RECT>::with_capacity(32)));

    let game_window = unsafe { FindWindowA(None, s!("AssaultCube")) }?;

    let mut window_info = WINDOWINFO::default();
    unsafe { GetWindowInfo(game_window, &mut window_info) }?;

    let draw_rect_list_clone = Arc::clone(&draw_rect_list);
    thread::spawn(move || {
        let mut overlay = overlay::Overlay::new(
            window_info.rcClient.left,
            window_info.rcClient.top,
            window_info.rcClient.right,
            window_info.rcClient.bottom,
            draw_rect_list_clone,
            true,
        )
        .unwrap();
        let _ = overlay.run();
    });

    let viewport = Viewport {
        width: window_info.rcClient.right - window_info.rcClient.left,
        height: window_info.rcClient.bottom - window_info.rcClient.top,
    };

    let module_base_addr = unsafe { GetModuleHandleA(s!("ac_client.exe")).map(|h| h.0 as u32) }?;

    let entity_list_base_addr = ac_client::read::<u32>(module_base_addr, ac_client::ENTITY_LIST);

    esp_loop(
        module_base_addr,
        entity_list_base_addr,
        viewport,
        draw_rect_list,
    );

    Ok(())
}

fn esp_loop(
    module_base_addr: u32,
    entity_list_base_addr: u32,
    viewport: Viewport,
    draw_rect_list: Arc<RwLock<Vec<RECT>>>,
) {
    let mut last_tick = Instant::now();

    loop {
        let player_count = ac_client::read::<u32>(module_base_addr, ac_client::PLAYER_COUNT);
        let view_matrix = ac_client::read::<[f32; 16]>(module_base_addr, ac_client::VIEW_MATRIX);

        let boxes: Vec<RECT> = (1..player_count)
            .filter_map(|index| {
                let entity = ac_client::read_entity(entity_list_base_addr, index);
                esp::screen_box(&entity, view_matrix, viewport)
            })
            .collect();

        if let Ok(mut draw_rect_list) = draw_rect_list.write() {
            draw_rect_list.clear();
            draw_rect_list.extend(boxes);
        }

        let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());
        thread::sleep(timeout);
        last_tick = Instant::now();
    }
}

#[unsafe(no_mangle)]
extern "system" fn DllMain(_dll_module: HINSTANCE, call_reason: u32, _reserved: *mut ()) -> bool {
    if call_reason == DLL_PROCESS_ATTACH {
        thread::spawn(move || {
            let _ = run();
        });
    }
    true
}
