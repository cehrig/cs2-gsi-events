use crate::error::Error;
use crate::windows::utility::to_wstring;
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use windows::core::{Interface, HSTRING, PCWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::UI::WindowsAndMessaging::*;

const CLASS_NAME: &str = "rust_overlay";

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub struct Window {
    width: i32,
    height: i32,
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    swap_chain: IDXGISwapChain1,
    comp_dev: IDCompositionDevice,
    target: IDCompositionTarget,
    visual: IDCompositionVisual,
    rtv: ID3D11RenderTargetView,
    d2d: ID2D1DeviceContext5,
    text_format: IDWriteTextFormat,
}

impl Window {
    pub fn events(&self, mut rx: Receiver<String>) -> Result<(), Error> {
        unsafe {
            // Render Loop
            let mut msg = MSG::default();

            let mut m = 0.0;
            loop {
                while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).into() {
                    if msg.message == WM_QUIT {
                        return Ok(());
                    }
                    let _ = TranslateMessage(&msg);
                    let _ = DispatchMessageW(&msg);
                }

                let Some(text) = rx.blocking_recv() else {
                    return Ok(());
                };

                m = m + 0.001;
                if m >= 1.0 {
                    m = 0.0;
                }

                let clear_color = [0.0, 0.0, 0.0, 0.00];

                self.context.ClearRenderTargetView(&self.rtv, &clear_color);

                self.d2d.BeginDraw();
                let brush = self.d2d.CreateSolidColorBrush(
                    &D2D1_COLOR_F {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.9,
                    },
                    None,
                )?;

                let rect = D2D_RECT_F {
                    left: 0.0,
                    top: 0.0,
                    right: self.width as f32 - 100.0,
                    bottom: self.height as f32 - 100.0,
                };

                let text = to_wstring(format!("{}", text).as_str());

                self.d2d.DrawText(
                    &text,
                    &self.text_format,
                    &rect,
                    &brush,
                    None,
                    0,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                    DWRITE_MEASURING_MODE_NATURAL,
                );

                self.d2d.EndDraw(None, None)?;

                self.swap_chain.Present(0, DXGI_PRESENT(0)).ok()?;
                sleep(Duration::from_millis(20));
            }
        }
    }
}

pub fn setup() -> Result<Window, Error> {
    let width = get_system_metric(SM_CXSCREEN)?;
    let height = get_system_metric(SM_CYSCREEN)?;

    let module = create_class()?;
    let hwnd = create_window(module, width, height)?;
    let (device, context) = create_device()?;
    let swap_chain = create_swap_chain(&device, width, height)?;
    let (comp_dev, target, visual, rtv) = create_visuals(hwnd, &device, &swap_chain)?;
    let (d2d, _, text_format) = create_d2d(&device, &swap_chain)?;

    let window = Window {
        width,
        height,
        device,
        context,
        swap_chain,
        comp_dev,
        target,
        visual,
        rtv,
        d2d,
        text_format,
    };

    Ok(window)
}

fn get_system_metric(metric: SYSTEM_METRICS_INDEX) -> Result<i32, Error> {
    let metrics = unsafe { GetSystemMetrics(metric) };

    match metrics {
        0 => Err(Error::GetSystemMetrics),
        n => Ok(n),
    }
}

fn create_class() -> Result<HMODULE, Error> {
    let hinstance = unsafe { GetModuleHandleW(None)? };
    let class_name = to_wstring(CLASS_NAME);

    let wc = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        style: CS_NOCLOSE,
        lpfnWndProc: Some(wndproc),
        hInstance: hinstance.into(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
        ..Default::default()
    };

    if unsafe { RegisterClassExW(&wc) } == 0 {
        return Err(Error::WindowsClassCreate);
    }

    Ok(hinstance)
}

fn create_window(module: HMODULE, width: i32, height: i32) -> Result<HWND, Error> {
    let class_name = to_wstring(CLASS_NAME);

    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW,
            PCWSTR(class_name.as_ptr()),
            PCWSTR(class_name.as_ptr()),
            WS_POPUP,
            0,
            0,
            width,
            height,
            None,
            None,
            Some(module.into()),
            None,
        )?
    };

    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
    };

    Ok(hwnd)
}

fn create_device() -> Result<(ID3D11Device, ID3D11DeviceContext), Error> {
    let mut device: Option<ID3D11Device> = None;
    let mut context: Option<ID3D11DeviceContext> = None;

    let feature_levels = [D3D_FEATURE_LEVEL_11_0];

    unsafe {
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            Some(&feature_levels),
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        )?
    };

    let device = device.ok_or_else(|| Error::D3DeviceMissing)?;
    let context = context.ok_or_else(|| Error::D3ContextMissing)?;

    Ok((device, context))
}

fn create_swap_chain(
    device: &ID3D11Device,
    width: i32,
    height: i32,
) -> Result<IDXGISwapChain1, Error> {
    let dxgi_device: IDXGIDevice = device.cast()?;
    let adapter = unsafe { dxgi_device.GetAdapter()? };
    let factory: IDXGIFactory2 = unsafe { adapter.GetParent()? };

    let swap_desc = DXGI_SWAP_CHAIN_DESC1 {
        Width: width as u32,
        Height: height as u32,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: 2,
        SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
        AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        ..Default::default()
    };

    let swap_chain = unsafe { factory.CreateSwapChainForComposition(device, &swap_desc, None)? };

    Ok(swap_chain)
}

fn create_visuals(
    hwnd: HWND,
    device: &ID3D11Device,
    swap_chain: &IDXGISwapChain1,
) -> Result<
    (
        IDCompositionDevice,
        IDCompositionTarget,
        IDCompositionVisual,
        ID3D11RenderTargetView,
    ),
    Error,
> {
    let comp_device: IDCompositionDevice = unsafe { DCompositionCreateDevice(None)? };
    let target = unsafe { comp_device.CreateTargetForHwnd(hwnd, true)? };
    let visual = unsafe { comp_device.CreateVisual()? };

    unsafe { visual.SetContent(swap_chain)? };
    unsafe { target.SetRoot(&visual)? };
    unsafe { comp_device.Commit()? };

    let backbuffer: ID3D11Texture2D = unsafe { swap_chain.GetBuffer(0)? };
    let mut rtv: Option<ID3D11RenderTargetView> = None;
    unsafe { device.CreateRenderTargetView(&backbuffer, None, Some(&mut rtv))? };

    Ok((
        comp_device,
        target,
        visual,
        rtv.ok_or_else(|| Error::D3RenderTargetMissing)?,
    ))
}

fn create_d2d(
    device: &ID3D11Device,
    swap_chain: &IDXGISwapChain1,
) -> Result<(ID2D1DeviceContext5, ID2D1Bitmap1, IDWriteTextFormat), Error> {
    let dxgi_device: IDXGIDevice = device.cast()?;

    let d2d_factory: ID2D1Factory1 =
        unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)? };

    let d2d_device: ID2D1Device = unsafe { d2d_factory.CreateDevice(&dxgi_device)? };

    let d2d_context: ID2D1DeviceContext5 = unsafe {
        d2d_device
            .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?
            .cast()?
    };

    let surface: IDXGISurface = unsafe { swap_chain.GetBuffer(0)? };

    let bitmap_props = D2D1_BITMAP_PROPERTIES1 {
        pixelFormat: D2D1_PIXEL_FORMAT {
            format: DXGI_FORMAT_B8G8R8A8_UNORM,
            alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
        },
        dpiX: 96.0,
        dpiY: 96.0,
        bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
        ..Default::default()
    };

    let bitmap = unsafe { d2d_context.CreateBitmapFromDxgiSurface(&surface, Some(&bitmap_props))? };

    unsafe { d2d_context.SetTarget(&bitmap) };

    let dwrite: IDWriteFactory = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)? };

    let format: IDWriteTextFormat = unsafe {
        dwrite.CreateTextFormat(
            &HSTRING::from("Consolas"),
            None,
            DWRITE_FONT_WEIGHT_BOLD,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            48.0,
            &HSTRING::from("en-us"),
        )?
    };

    unsafe { format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)? };
    unsafe { format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)? };

    Ok((d2d_context, bitmap, format))
}
