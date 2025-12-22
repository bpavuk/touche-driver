use crate::data::{ToucheData, events::ToucheEvent};
use std::{error::Error, io};

pub mod graphics_tablet;
pub mod touchpad;

pub trait DeviceSink {
    fn emit(&mut self, touche_data: &[ToucheEvent]) -> Result<(), io::Error>;

    fn init(&mut self, width: i32, height: i32) -> Result<(), Box<dyn std::error::Error>>;

    fn combine<T: DeviceSink>(self, other: T) -> CombinedSink<Self, T>
    where 
        Self: Sized
    {
        CombinedSink {
            a: self,
            b: other,
        }
    }
}

pub struct CombinedSink<A: DeviceSink, B: DeviceSink> {
    pub a: A,
    pub b: B,
}

impl<A: DeviceSink, B: DeviceSink> DeviceSink for CombinedSink<A, B> {
    fn emit(&mut self, touche_data: &[ToucheEvent]) -> Result<(), io::Error> {
        self.a.emit(touche_data)?;
        self.b.emit(touche_data)?;

        Ok(())
    }

    fn init(&mut self, width: i32, height: i32) -> Result<(), Box<dyn std::error::Error>> {
        self.a.init(width, height)?;
        self.b.init(width, height)?;

        Ok(())
    }
}

pub trait ToucheSource {
    fn blocking_read(&mut self) -> Result<Vec<ToucheData>, Box<dyn Error>>;
}
