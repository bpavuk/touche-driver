mod state;

use core::result::Result;
use std::error::Error;

use crate::{
    data::{ToucheData, events::ToucheEvent},
    devices::{
        DeviceSink, ToucheSource,
    },
    driver::state::DriverState,
};

use log::{info, warn};

pub struct Driver<D: DeviceSink, S: ToucheSource> {
    pub device_sink: D,
    pub touche_source: S,
    state: DriverState,
}

impl<D: DeviceSink, S: ToucheSource> Driver<D, S> {
    pub fn new(device_sink: D, touche_source: S) -> Driver<D, S> {
        Driver {
            device_sink,
            touche_source,
            state: DriverState::Uninitialized,
        }
    }

    pub fn tick(&mut self) -> Result<(), Box<dyn Error>> {
        let data: Vec<ToucheData> = self.touche_source.blocking_read()?;

        let events: Vec<_> = data
            .iter()
            .filter_map(|data| match data {
                ToucheData::ScreenSize { x, y } => match self.state {
                    DriverState::Initialized => {
                        warn!(
                            "Received a new screen event, although the driver is already initialized"
                        );
                        None
                    }
                    DriverState::Uninitialized => {
                        info!("Initializing the driver...");

                        let _ = self.device_sink.init(*x, *y);
                        self.state = DriverState::Initialized;

                        None
                    },
                },
                ToucheData::StylusFrame {
                    x,
                    y,
                    pressed,
                    pressure,
                } => Some(ToucheEvent::Stylus {
                    x: *x, y: *y, pressed: *pressed, pressure: *pressure
                }),
                ToucheData::TouchFrame {
                    x,
                    y,
                    touch_id,
                    pressed,
                } => Some(ToucheEvent::Touch { x: *x, y: *y, touch_id: *touch_id, pressed: *pressed }),
                ToucheData::ButtonFrame { button_id, pressed } => Some(ToucheEvent::Button { button_id: *button_id, pressed: *pressed }),
                ToucheData::Action(_) => {
                    self.state = DriverState::Uninitialized;

                    None
                },
            })
            .collect::<Vec<ToucheEvent>>();

        if !events.is_empty() {
            self.device_sink.emit(&events[..])?;
        }

        Ok(())
    }

    pub fn start_loop(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            self.tick()?;
        }
    }
}
