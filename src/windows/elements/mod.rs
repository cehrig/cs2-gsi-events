use crate::error::Error;
use crate::windows::window::Window;

pub mod text;

pub use text::*;

pub trait Draw2D: Send + Sync + 'static {
    fn draw(&self, window: &Window) -> Result<(), Error>;
}

#[derive(PartialEq, Eq, Hash)]
pub struct ElementIdentifier(String);

impl ElementIdentifier {
    pub fn new(id: &str) -> Self {
        Self { 0: id.to_string() }
    }
}

#[derive(Clone)]
pub struct RGBA(f32, f32, f32, f32);

impl RGBA {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> RGBA {
        RGBA(r, g, b, a)
    }

    pub fn red(&self) -> f32 {
        self.0
    }

    pub fn green(&self) -> f32 {
        self.1
    }

    pub fn blue(&self) -> f32 {
        self.2
    }

    pub fn alpha(&self) -> f32 {
        self.3
    }
}

#[derive(Clone)]
pub enum PositionMode {
    Absolute,
    FromCenter,
}

#[derive(Clone)]
pub struct Position {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    mode: PositionMode,
}

impl Position {
    pub fn new(left: f32, top: f32, right: f32, bottom: f32, mode: PositionMode) -> Position {
        Self {
            left,
            top,
            right,
            bottom,
            mode,
        }
    }

    pub fn left(&self, total_width: i32) -> f32 {
        self.abs_rel(self.left, total_width)
    }

    pub fn top(&self, total_height: i32) -> f32 {
        self.abs_rel(self.top, total_height)
    }

    pub fn right(&self, total_width: i32) -> f32 {
        self.abs_rel(self.right, total_width)
    }

    pub fn bottom(&self, total_height: i32) -> f32 {
        self.abs_rel(self.bottom, total_height)
    }

    fn abs_rel(&self, val: f32, init: i32) -> f32 {
        match self.mode {
            PositionMode::Absolute => val,
            PositionMode::FromCenter => init as f32 / 2.0 + val,
        }
    }
}
