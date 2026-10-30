use std::sync::{Arc, RwLock};
use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct2D::Common::*, Win32::Graphics::Direct2D::*,
    Win32::Graphics::Direct3D::*, Win32::Graphics::Direct3D11::*,
    Win32::Graphics::DirectComposition::*, Win32::Graphics::Dxgi::Common::*,
    Win32::Graphics::Dxgi::*, Win32::Graphics::Gdi::*, Win32::UI::WindowsAndMessaging::*, core::*,
};
use windows_numerics::Vector2;

pub struct Overlay {
    handle: HWND,

    window_rect: RECT,
    draw_rect_list: Arc<RwLock<Vec<RECT>>>,
    draw_bottom_line_flag: bool,

    factory: ID2D1Factory1,
    style: ID2D1StrokeStyle1,

    composition: Option<IDCompositionDevice>,
    composition_target: Option<IDCompositionTarget>,
    visual: Option<IDCompositionVisual>,
    target: Option<ID2D1DeviceContext>,
    swapchain: Option<IDXGISwapChain1>,
    brush: Option<ID2D1SolidColorBrush>,
}

impl Drop for Overlay {
    fn drop(&mut self) {
        self.destroy_window();
    }
}

impl Overlay {
    pub fn destroy_window(&mut self) {
        unsafe {
            if !self.handle.is_invalid() {
                _ = DestroyWindow(self.handle);
                self.handle = HWND::default();
            }
            _ = UnregisterClassA(s!("ezOverlay"), None);
        }
    }
}

impl Overlay {
    pub fn new(
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        draw_rect_list: Arc<RwLock<Vec<RECT>>>,
        draw_bottom_line_flag: bool,
    ) -> Result<Self> {
        let factory = create_factory()?;
        let style = create_style(&factory)?;

        Ok(Overlay {
            handle: Default::default(),
            window_rect: RECT {
                left,
                top,
                right,
                bottom,
            },
            draw_rect_list,
            draw_bottom_line_flag,
            factory,
            style,
            composition: None,
            composition_target: None,
            visual: None,
            target: None,
            swapchain: None,
            brush: None,
        })
    }

    pub fn run(&mut self, stop: impl Fn() -> bool) -> Result<()> {
        unsafe {
            let window_class_name = s!("ezOverlay");

            let window_class = WNDCLASSA {
                lpszClassName: window_class_name,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wndproc),
                ..Default::default()
            };
            let atom = RegisterClassA(&window_class);
            debug_assert!(atom != 0);

            let window_width = self.window_rect.right - self.window_rect.left;
            let window_height = self.window_rect.bottom - self.window_rect.top;

            let handle = CreateWindowExA(
                WS_EX_TOPMOST
                    | WS_EX_TRANSPARENT
                    | WS_EX_LAYERED
                    | WS_EX_NOREDIRECTIONBITMAP
                    | WS_EX_NOACTIVATE
                    | WS_EX_TOOLWINDOW,
                window_class_name,
                s!("EZ Overlay"),
                WS_POPUP | WS_VISIBLE,
                self.window_rect.left,
                self.window_rect.top,
                window_width,
                window_height,
                None,
                None,
                None,
                Some(self as *mut _ as _),
            )?;

            debug_assert!(!handle.is_invalid());
            debug_assert!(handle == self.handle);

            let mut message = MSG::default();
            loop {
                if stop() {
                    return Ok(());
                }

                self.render()?;

                while PeekMessageA(&mut message, None, 0, 0, PM_REMOVE).into() {
                    if message.message == WM_QUIT {
                        return Ok(());
                    }
                    DispatchMessageA(&message);
                }
            }
        }
    }

    fn render(&mut self) -> Result<()> {
        if self.target.is_none() {
            let device = create_device()?;
            let target = create_render_target(&self.factory, &device)?;

            let width = (self.window_rect.right - self.window_rect.left) as u32;
            let height = (self.window_rect.bottom - self.window_rect.top) as u32;
            let swapchain = create_swapchain(&device, width, height)?;
            create_swapchain_bitmap(&swapchain, &target)?;

            let (composition, composition_target, visual) =
                create_composition(&device, &swapchain, self.handle)?;

            self.brush = create_brush(&target).ok();
            self.target = Some(target);
            self.swapchain = Some(swapchain);
            self.composition = Some(composition);
            self.composition_target = Some(composition_target);
            self.visual = Some(visual);
        }

        let target = self.target.as_ref().unwrap();
        unsafe { target.BeginDraw() };
        self.draw(target)?;

        unsafe {
            target.EndDraw(None, None)?;
        }

        if let Err(error) = self.present(1, DXGI_PRESENT(0))
            && error.code() != DXGI_STATUS_OCCLUDED
        {
            self.release_device();
        }

        Ok(())
    }

    fn release_device(&mut self) {
        self.target = None;
        self.swapchain = None;
        self.composition = None;
        self.composition_target = None;
        self.visual = None;
        self.release_device_resources();
    }

    fn release_device_resources(&mut self) {
        self.brush = None;
    }

    fn present(&self, sync: u32, flags: DXGI_PRESENT) -> Result<()> {
        unsafe { self.swapchain.as_ref().unwrap().Present(sync, flags).ok() }
    }

    fn draw(&self, target: &ID2D1DeviceContext) -> Result<()> {
        unsafe {
            target.Clear(Some(&D2D1_COLOR_F {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }));

            let brush = self.brush.as_ref().unwrap();
            let draw_rect_list = {
                let draw_rect_list_lock = self.draw_rect_list.read().unwrap();
                draw_rect_list_lock.clone()
            };
            draw_rect_list.iter().for_each(|rect| {
                target.DrawRectangle(
                    &D2D_RECT_F {
                        left: rect.left as f32,
                        top: rect.top as f32,
                        right: rect.right as f32,
                        bottom: rect.bottom as f32,
                    },
                    brush,
                    2.0,
                    &self.style,
                );
                if self.draw_bottom_line_flag {
                    let rect_width = rect.right - rect.left;
                    target.DrawLine(
                        Vector2 {
                            X: (self.window_rect.right / 2) as f32,
                            Y: self.window_rect.bottom as f32,
                        },
                        Vector2 {
                            X: (rect.left + rect_width / 2) as f32,
                            Y: rect.bottom as f32,
                        },
                        brush,
                        2.0,
                        &self.style,
                    );
                }
            });
        }

        Ok(())
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match message {
                WM_PAINT => {
                    let mut ps = PAINTSTRUCT::default();
                    BeginPaint(self.handle, &mut ps);
                    self.render().unwrap();
                    _ = EndPaint(self.handle, &ps);
                    LRESULT(0)
                }
                WM_DISPLAYCHANGE => {
                    self.render().unwrap();
                    LRESULT(0)
                }
                WM_NCHITTEST => LRESULT(HTTRANSPARENT as isize),
                WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                _ => DefWindowProcA(self.handle, message, wparam, lparam),
            }
        }
    }

    extern "system" fn wndproc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            if message == WM_NCCREATE {
                let cs = lparam.0 as *const CREATESTRUCTA;
                let this = (*cs).lpCreateParams as *mut Self;
                (*this).handle = window;

                SetWindowLongPtrA(window, GWLP_USERDATA, this as _);
            } else {
                let this = GetWindowLongPtrA(window, GWLP_USERDATA) as *mut Self;

                if !this.is_null() {
                    return (*this).message_handler(message, wparam, lparam);
                }
            }

            DefWindowProcA(window, message, wparam, lparam)
        }
    }
}

fn create_brush(target: &ID2D1DeviceContext) -> Result<ID2D1SolidColorBrush> {
    let color = D2D1_COLOR_F {
        r: 0.15,
        g: 1.0,
        b: 0.35,
        a: 1.0,
    };

    let properties = D2D1_BRUSH_PROPERTIES {
        opacity: 1.0,
        ..Default::default()
    };

    unsafe { target.CreateSolidColorBrush(&color, Some(&properties)) }
}

fn create_factory() -> Result<ID2D1Factory1> {
    let mut options = D2D1_FACTORY_OPTIONS::default();

    if cfg!(debug_assertions) {
        options.debugLevel = D2D1_DEBUG_LEVEL_INFORMATION;
    }

    unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, Some(&options)) }
}

fn create_style(factory: &ID2D1Factory1) -> Result<ID2D1StrokeStyle1> {
    let props = D2D1_STROKE_STYLE_PROPERTIES1 {
        startCap: D2D1_CAP_STYLE_ROUND,
        endCap: D2D1_CAP_STYLE_TRIANGLE,
        ..Default::default()
    };

    unsafe { factory.CreateStrokeStyle(&props, None) }
}

fn create_device_with_type(drive_type: D3D_DRIVER_TYPE) -> Result<ID3D11Device> {
    let mut flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;

    if cfg!(debug_assertions) {
        flags |= D3D11_CREATE_DEVICE_DEBUG;
    }

    let mut device = None;

    unsafe {
        D3D11CreateDevice(
            None,
            drive_type,
            HMODULE::default(),
            flags,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            None,
        )
        .map(|()| device.unwrap())
    }
}

fn create_device() -> Result<ID3D11Device> {
    let mut result = create_device_with_type(D3D_DRIVER_TYPE_HARDWARE);

    if let Err(err) = &result
        && err.code() == DXGI_ERROR_UNSUPPORTED
    {
        result = create_device_with_type(D3D_DRIVER_TYPE_WARP);
    }

    result
}

fn create_render_target(
    factory: &ID2D1Factory1,
    device: &ID3D11Device,
) -> Result<ID2D1DeviceContext> {
    unsafe {
        let d2device = factory.CreateDevice(&device.cast::<IDXGIDevice>()?)?;

        let target = d2device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;

        target.SetUnitMode(D2D1_UNIT_MODE_DIPS);

        Ok(target)
    }
}

fn get_dxgi_factory(device: &ID3D11Device) -> Result<IDXGIFactory2> {
    let dxdevice = device.cast::<IDXGIDevice>()?;
    unsafe { dxdevice.GetAdapter()?.GetParent() }
}

fn create_swapchain_bitmap(swapchain: &IDXGISwapChain1, target: &ID2D1DeviceContext) -> Result<()> {
    let surface: IDXGISurface = unsafe { swapchain.GetBuffer(0)? };

    let props = D2D1_BITMAP_PROPERTIES1 {
        pixelFormat: D2D1_PIXEL_FORMAT {
            format: DXGI_FORMAT_B8G8R8A8_UNORM,
            alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
        },
        bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
        ..Default::default()
    };

    unsafe {
        let bitmap = target.CreateBitmapFromDxgiSurface(&surface, Some(&props))?;
        target.SetTarget(&bitmap);
    };

    Ok(())
}

fn create_composition(
    device: &ID3D11Device,
    swapchain: &IDXGISwapChain1,
    window: HWND,
) -> Result<(
    IDCompositionDevice,
    IDCompositionTarget,
    IDCompositionVisual,
)> {
    unsafe {
        let dxgi_device: IDXGIDevice = device.cast()?;
        let composition: IDCompositionDevice = DCompositionCreateDevice(&dxgi_device)?;
        let target = composition.CreateTargetForHwnd(window, true)?;
        let visual = composition.CreateVisual()?;

        visual.SetContent(swapchain)?;
        target.SetRoot(&visual)?;
        composition.Commit()?;

        Ok((composition, target, visual))
    }
}

fn create_swapchain(device: &ID3D11Device, width: u32, height: u32) -> Result<IDXGISwapChain1> {
    let factory = get_dxgi_factory(device)?;

    let props = DXGI_SWAP_CHAIN_DESC1 {
        Width: width,
        Height: height,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: 2,
        SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
        AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
        ..Default::default()
    };

    unsafe { factory.CreateSwapChainForComposition(device, &props, None) }
}
