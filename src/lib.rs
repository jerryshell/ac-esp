mod ac_client;
mod esp;
pub mod overlay;

use anyhow::Result;
use esp::Viewport;
use std::{
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, AtomicIsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::*,
        System::{LibraryLoader::*, SystemServices::*, Threading::*},
        UI::WindowsAndMessaging::*,
    },
    core::*,
};

const FRAME_RATE: u64 = 60;
const TICK_RATE: Duration = Duration::from_millis(1000 / FRAME_RATE);

static SHUTDOWN: AtomicBool = AtomicBool::new(false);
static GAME_DONE: AtomicIsize = AtomicIsize::new(0);
static OVERLAY_DONE: AtomicIsize = AtomicIsize::new(0);

fn run() -> Result<()> {
    let draw_rect_list = Arc::new(RwLock::new(Vec::<RECT>::with_capacity(32)));

    let game_window = unsafe { FindWindowA(None, s!("AssaultCube")) }?;

    let mut window_info = WINDOWINFO::default();
    unsafe { GetWindowInfo(game_window, &mut window_info) }?;

    let draw_rect_list_clone = Arc::clone(&draw_rect_list);
    thread::spawn(move || {
        if let Ok(mut overlay) = overlay::Overlay::new(
            window_info.rcClient.left,
            window_info.rcClient.top,
            window_info.rcClient.right,
            window_info.rcClient.bottom,
            draw_rect_list_clone,
            true,
        ) {
            let _ = overlay.run(|| SHUTDOWN.load(Ordering::SeqCst));
            overlay.destroy_window();
            std::mem::forget(overlay);
        }

        signal(OVERLAY_DONE.load(Ordering::SeqCst));
        unsafe { ExitThread(0) }
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

    while !SHUTDOWN.load(Ordering::SeqCst) {
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

fn signal(handle: isize) {
    let handle = HANDLE(handle as *mut _);

    if !handle.0.is_null() {
        _ = unsafe { SetEvent(handle) };
    }
}

fn wait(handle: isize) {
    let handle = HANDLE(handle as *mut _);

    if !handle.0.is_null() {
        _ = unsafe { WaitForSingleObject(handle, 2000) };
    }
}

#[unsafe(no_mangle)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, reserved: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            _ = unsafe { DisableThreadLibraryCalls(HMODULE(dll_module.0)) };

            let game_done = unsafe { CreateEventW(None, true, false, PCWSTR::null()) };
            GAME_DONE.store(
                game_done.map(|handle| handle.0 as isize).unwrap_or(0),
                Ordering::SeqCst,
            );

            let overlay_done = unsafe { CreateEventW(None, true, false, PCWSTR::null()) };
            OVERLAY_DONE.store(
                overlay_done.map(|handle| handle.0 as isize).unwrap_or(0),
                Ordering::SeqCst,
            );

            thread::spawn(|| {
                let _ = run();
                signal(GAME_DONE.load(Ordering::SeqCst));
                unsafe { ExitThread(0) }
            });
        }
        DLL_PROCESS_DETACH => {
            SHUTDOWN.store(true, Ordering::SeqCst);

            if reserved.is_null() {
                wait(GAME_DONE.load(Ordering::SeqCst));
                wait(OVERLAY_DONE.load(Ordering::SeqCst));
            }
        }
        _ => {}
    }
    true
}
