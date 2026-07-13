use crate::error::Error;
use crate::overlay::WindowEvent;
use tokio::sync::mpsc::Receiver;

pub struct Window;

pub fn setup() -> Result<Window, Error> {
    Ok(Window)
}

impl Window {
    pub fn events(&mut self, _: Receiver<WindowEvent>) -> Result<(), Error> {
        Ok(())
    }
}
