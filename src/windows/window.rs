use crate::error::Error;
use crate::windows::elements::{Draw2D, ElementIdentifier};
use crate::windows::events::WindowEvent;
use crate::windows::utility::to_wstring;
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;
use windows::core::{Interface, PCWSTR};
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

pub struct Renderer {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    swap_chain: IDXGISwapChain1,
    comp_dev: IDCompositionDevice,
    rtv: ID3D11RenderTargetView,
    pub(crate) d2d: ID2D1DeviceContext5,
    pub(crate) write_factory: IDWriteFactory,
}

struct Composition {
    target: IDCompositionTarget,
    visual: IDCompositionVisual,
}

pub struct Window {
    width: i32,
    height: i32,
    pub(crate) renderer: Renderer,
    composition: Composition,
    elements_2d: HashMap<ElementIdentifier, Box<dyn Draw2D>>,
}

impl Renderer {
    fn clear_frame(&self) {
        unsafe {
            self.context
                .ClearRenderTargetView(&self.rtv, &[0.0, 0.0, 0.0, 0.00])
        };
    }

    fn start_frame(&self, window: &Window) -> Result<(), Error> {
        if window.elements_2d.is_empty() {
            return Ok(());
        }

        unsafe { self.d2d.BeginDraw() };

        for (_, element) in window.elements_2d.iter() {
            element.draw(window)?;
        }

        Ok(())
    }

    fn end_frame(&self, window: &Window) -> Result<(), Error> {
        if !window.elements_2d.is_empty() {
            unsafe { self.d2d.EndDraw(None, None)? }
        }

        unsafe { self.swap_chain.Present(1, DXGI_PRESENT(0)).ok()? }

        Ok(())
    }
}

impl Window {
    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn events(&mut self, mut rx: Receiver<WindowEvent>) -> Result<(), Error> {
        unsafe {
            // Render Loop
            let mut msg = MSG::default();

            loop {
                while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).into() {
                    if msg.message == WM_QUIT {
                        return Ok(());
                    }
                    let _ = TranslateMessage(&msg);
                    let _ = DispatchMessageW(&msg);
                }

                let Some(event) = rx.blocking_recv() else {
                    return Ok(());
                };

                match event {
                    WindowEvent::Add2DElement((id, element)) => {
                        self.elements_2d.insert(id, element);
                        continue;
                    }
                    WindowEvent::Draw => {}
                }

                self.renderer.clear_frame();
                self.renderer.start_frame(self)?;
                self.renderer.end_frame(self)?;
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
    let (d2d, write_factory) = create_d2d(&device, &swap_chain)?;

    let renderer = Renderer {
        device,
        context,
        swap_chain,
        comp_dev,
        rtv,
        d2d,
        write_factory,
    };

    let composition = Composition { target, visual };

    let window = Window {
        width,
        height,
        renderer,
        composition,
        elements_2d: Default::default(),
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
) -> Result<(ID2D1DeviceContext5, IDWriteFactory), Error> {
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

    Ok((d2d_context, dwrite))
}
