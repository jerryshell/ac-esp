use std::time::{Duration, Instant};
use windows::Win32::Foundation::RECT;

#[test]
fn overlay_test() {
    let start = Instant::now();

    std::thread::spawn(move || {
        let mut overlay = ac_esp::overlay::Overlay::new(
            0,
            0,
            1920,
            1080,
            Box::new(move || {
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
