use crate::overlay::elements::{Draw2D, ElementIdentifier};

pub enum WindowEvent {
    Add2DElement((ElementIdentifier, Box<dyn Draw2D>)),
    Draw,
}

impl WindowEvent {
    pub fn add_2d_element(id: &str, element: impl Draw2D) -> Self {
        Self::Add2DElement((ElementIdentifier::new(id), Box::new(element)))
    }
}
