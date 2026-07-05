use crate::error::Error;
use crate::windows::elements::{Draw2D, Position, RGBA};
use crate::windows::utility::to_wstring;
use crate::windows::window::Window;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows_core::HSTRING;

pub struct TextElement {
    inner: Mutex<TextElementInner>,
}

#[derive(Clone)]
struct TextElementInner {
    color: RGBA,
    position: Position,
    format: TextFormat,
    cached_format: Option<IDWriteTextFormat>,
    text: Option<String>,
}

#[derive(Clone)]
pub struct TextFormat {
    family: HSTRING,
    size: f32,
}

impl TextElement {
    pub fn new(
        color: RGBA,
        position: Position,
        format: TextFormat,
        text: Option<String>,
    ) -> Arc<TextElement> {
        Arc::new(TextElement {
            inner: Mutex::new(TextElementInner::new(color, position, format, text)),
        })
    }

    fn lock(&self) -> Result<impl DerefMut<Target = TextElementInner>, Error> {
        let inner = self.inner.lock().map_err(|_| Error::LockFailed)?;

        Ok(inner)
    }

    pub fn set_text(&self, text: String) -> Result<(), Error> {
        let mut lock = self.lock()?;
        lock.text = Some(text);

        Ok(())
    }

    pub fn clear_text(&self) -> Result<(), Error> {
        let mut lock = self.lock()?;
        lock.text = None;

        Ok(())
    }

    pub fn set_color(&self, color: RGBA) -> Result<(), Error> {
        let mut lock = self.lock()?;
        lock.color = color;

        Ok(())
    }

    pub fn set_format(&self, format: TextFormat) -> Result<(), Error> {
        let mut lock = self.lock()?;
        lock.format = format;
        lock.cached_format = None;

        Ok(())
    }

    pub fn set_cached_format(&self, format: IDWriteTextFormat) -> Result<(), Error> {
        let mut lock = self.lock()?;
        lock.cached_format = Some(format);

        Ok(())
    }
}

impl TextElementInner {
    fn new(color: RGBA, position: Position, format: TextFormat, text: Option<String>) -> Self {
        Self {
            color,
            position,
            format,
            cached_format: None,
            text,
        }
    }
}

impl TextFormat {
    pub fn new(family: &str, size: f32) -> Self {
        Self {
            family: HSTRING::from(family),
            size,
        }
    }

    fn get_cached_format(&self, window: &Window) -> Result<IDWriteTextFormat, Error> {
        let format: IDWriteTextFormat = unsafe {
            window.renderer.write_factory.CreateTextFormat(
                &self.family,
                None,
                DWRITE_FONT_WEIGHT_BOLD,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                self.size,
                &HSTRING::from("en-us"),
            )?
        };

        unsafe { format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING)? };
        unsafe { format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_NEAR)? };

        Ok(format)
    }
}

impl Draw2D for Arc<TextElement> {
    fn draw(&self, window: &Window) -> Result<(), Error> {
        // Clone Element instead of holding the lock while drawing
        let state = {
            let lock = self.inner.lock().map_err(|_| Error::LockFailed)?;
            lock.clone()
        };

        let Some(text) = state.text.as_deref() else {
            return Ok(());
        };

        let text = to_wstring(text);

        let brush = unsafe {
            window.renderer.d2d.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: state.color.red(),
                    g: state.color.green(),
                    b: state.color.blue(),
                    a: state.color.alpha(),
                },
                None,
            )?
        };

        let rect = D2D_RECT_F {
            left: state.position.left(window.width()),
            top: state.position.top(window.height()),
            right: state.position.right(window.width()),
            bottom: state.position.bottom(window.height()),
        };

        let cached_format;
        let format = match &state.cached_format {
            None => {
                let format = state.format.get_cached_format(window)?;
                self.set_cached_format(format.clone())?;

                cached_format = Some(format.clone());
                cached_format.as_ref().unwrap()
            }
            Some(format) => format,
        };

        unsafe {
            window.renderer.d2d.DrawText(
                &text,
                format,
                &rect,
                &brush,
                None,
                0,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            )
        };

        Ok(())
    }
}
