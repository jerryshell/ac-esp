#[test]
fn overlay_test() {
    let rect_list = std::sync::Arc::new(std::sync::RwLock::new(vec![
        windows::Win32::Foundation::RECT {
            left: 0,
            top: 0,
            right: 100,
            bottom: 100,
        },
        windows::Win32::Foundation::RECT {
            left: 123,
            top: 456,
            right: 789,
            bottom: 666,
        },
    ]));

    {
        let rect_list = rect_list.clone();
        std::thread::spawn(move || {
            let mut overlay =
                ac_esp::overlay::Overlay::new(0, 0, 1920, 1080, rect_list, true).unwrap();
            overlay.run().unwrap();
        });
    }

    const FRAME_RATE: u64 = 60;
    const TICK_RATE: std::time::Duration = std::time::Duration::from_millis(1000 / FRAME_RATE);

    let mut frame_count = 0;
    let mut last_tick = std::time::Instant::now();
    loop {
        {
            let mut rect_list = rect_list.write().unwrap();
            rect_list.iter_mut().for_each(|rect| {
                rect.left += 1;
                rect.top += 1;
                rect.right += 1;
                rect.bottom += 1;
            });
        }

        std::thread::sleep(TICK_RATE.saturating_sub(last_tick.elapsed()));
        last_tick = std::time::Instant::now();

        frame_count += 1;
        if frame_count >= 500 {
            break;
        }
    }
}
