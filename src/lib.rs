mod ac_client;
mod esp;
pub mod overlay;

use anyhow::Result;
use esp::Viewport;
use std::{
    sync::atomic::{AtomicBool, AtomicIsize, Ordering},
    thread,
};
use windows::{
    Win32::{
        Foundation::*,
        System::{LibraryLoader::*, SystemServices::*, Threading::*},
        UI::WindowsAndMessaging::*,
    },
    core::*,
};

static SHUTDOWN: AtomicBool = AtomicBool::new(false);
static DONE: AtomicIsize = AtomicIsize::new(0);

fn run() -> Result<()> {
    let game_window = unsafe { FindWindowA(None, s!("AssaultCube")) }?;

    let module_base_addr = unsafe { GetModuleHandleA(s!("ac_client.exe")).map(|h| h.0 as u32) }?;
    let entity_list_base_addr =
        ac_client::read::<u32>(module_base_addr, ac_client::ENTITY_LIST).unwrap_or_default();

    let mut overlay = overlay::Overlay::new(
        game_window,
        Box::new(move |width, height| {
            esp_boxes(
                module_base_addr,
                entity_list_base_addr,
                Viewport { width, height },
            )
        }),
        true,
    )?;

    let _ = overlay.run(|| SHUTDOWN.load(Ordering::SeqCst));
    overlay.destroy_window();
    std::mem::forget(overlay);

    Ok(())
}

fn esp_boxes(module_base_addr: u32, entity_list_base_addr: u32, viewport: Viewport) -> Vec<RECT> {
    let player_count =
        ac_client::read::<u32>(module_base_addr, ac_client::PLAYER_COUNT).unwrap_or_default();
    let view_matrix =
        ac_client::read::<[f32; 16]>(module_base_addr, ac_client::VIEW_MATRIX).unwrap_or_default();

    (1..player_count.min(ac_client::MAX_ENTITIES))
        .filter_map(|index| {
            let entity = ac_client::read_entity(entity_list_base_addr, index)?;
            esp::screen_box(&entity, view_matrix, viewport)
        })
        .collect()
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

            let done = unsafe { CreateEventW(None, true, false, PCWSTR::null()) };
            DONE.store(
                done.map(|handle| handle.0 as isize).unwrap_or(0),
                Ordering::SeqCst,
            );

            thread::spawn(|| {
                let _ = run();
                signal(DONE.load(Ordering::SeqCst));
                unsafe { ExitThread(0) }
            });
        }
        DLL_PROCESS_DETACH => {
            SHUTDOWN.store(true, Ordering::SeqCst);

            if reserved.is_null() {
                wait(DONE.load(Ordering::SeqCst));
            }
        }
        _ => {}
    }
    true
}
