mod state;

use core::result::Result;
use std::{error::Error, vec};

use crate::{
    data::{events::ToucheEvent, Action, ToucheData},
    devices::{DeviceSink, ToucheSource},
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

        let mut events: Vec<ToucheEvent> = vec![];

        for data_point in data {
            match data_point {
                ToucheData::ScreenSize { x, y } => match self.state {
                    DriverState::Initialized => {
                        warn!(
                            "Received a new screen event, although the driver is already initialized"
                        );
                    }
                    DriverState::Uninitialized => {
                        info!("Initializing the driver...");

                        self.device_sink.init(x, y)?;
                        self.state = DriverState::Initialized;
                    }
                },
                ToucheData::StylusFrame {
                    x,
                    y,
                    pressed,
                    pressure,
                } => events.push(ToucheEvent::Stylus {
                    x,
                    y,
                    pressed,
                    pressure,
                }),
                ToucheData::TouchFrame {
                    x,
                    y,
                    touch_id,
                    pressed,
                } => events.push(ToucheEvent::Touch {
                    x,
                    y,
                    touch_id,
                    pressed,
                }),
                ToucheData::ButtonFrame { button_id, pressed } => {
                    events.push(ToucheEvent::Button { button_id, pressed })
                }
                ToucheData::Action(Action::Init) => {
                    self.state = DriverState::Uninitialized;
                }
            }
        }

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
