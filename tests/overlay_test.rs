use std::time::{Duration, Instant};
use windows::{
    Win32::{Foundation::*, UI::WindowsAndMessaging::*},
    core::*,
};

unsafe extern "system" fn default_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcA(hwnd, message, wparam, lparam) }
}

#[test]
fn overlay_test() {
    let start = Instant::now();

    let game = unsafe {
        let class = s!("overlayTestGame");
        let window_class = WNDCLASSA {
            lpszClassName: class,
            lpfnWndProc: Some(default_proc),
            ..Default::default()
        };
        RegisterClassA(&window_class);

        CreateWindowExA(
            WINDOW_EX_STYLE(0),
            class,
            s!("game"),
            WS_POPUP | WS_VISIBLE,
            0,
            0,
            1920,
            1080,
            None,
            None,
            None,
            None,
        )
        .unwrap()
        .0 as isize
    };

    std::thread::spawn(move || {
        let mut overlay = ac_esp::overlay::Overlay::new(
            HWND(game as *mut _),
            Box::new(move |_width, _height| {
                let offset = (start.elapsed().as_millis() as i32 * 60) / 1000;

                vec![
                    RECT {
                        left: offset,
                        top: offset,
                        right: offset + 100,
                        bottom: offset + 100,
                    },
                    RECT {
                        left: 123 + offset,
                        top: 456 + offset,
                        right: 789 + offset,
                        bottom: 666 + offset,
                    },
                ]
            }),
            true,
        )
        .unwrap();

        overlay.run(|| false).unwrap();
    });

    std::thread::sleep(Duration::from_millis(8300));
}
