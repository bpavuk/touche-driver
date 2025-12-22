mod state;

use core::result::Result;
use std::error::Error;

use crate::{
    aoa::AoaDevice,
    data::{Action, ToucheData, events::ToucheEvent},
    devices::{graphics_tablet::GraphicsTabletDevice, touchpad::TouchpadDevice},
};

use log::{error, info, warn};
use state::DriverState;

pub fn driver_loop(aoa_device: AoaDevice) -> Result<(), Box<dyn Error>> {
    let mut state = DriverState::Uninitialized;

    loop {
        let raw = aoa_device.read()?;
        let data: Vec<ToucheData> = serde_cbor::from_slice(&raw[..]).unwrap();

        let events: Vec<ToucheEvent> = data
            .iter()
            .filter_map(|data| match data {
                ToucheData::ScreenSize { x, y } => match state {
                    DriverState::Initialized {
                        tablet: _,
                        touchpad: _,
                    } => {
                        warn!("screen event sent, but driver is already initialized");
                        None
                    }
                    DriverState::Uninitialized => {
                        let touchetab = match GraphicsTabletDevice::new(*x, *y) {
                            Ok(tab) => tab,
                            Err(e) => {
                                error!("graphics tablet creation error! {}", e);
                                info!("error logs:\n{}", e);

                                return None;
                            }
                        };
                        let touchepad = match TouchpadDevice::new(*x, *y) {
                            Ok(pad) => pad,
                            Err(e) => {
                                error!("touchpad creation error! {}", e);
                                info!("error logs:\n{}", e);

                                return None;
                            }
                        };

                        state = DriverState::Initialized {
                            tablet: touchetab,
                            touchpad: touchepad,
                        };

                        None
                    }
                },
                ToucheData::StylusFrame {
                    x,
                    y,
                    pressed,
                    pressure,
                } => Some(ToucheEvent::Stylus {
                    x: *x,
                    y: *y,
                    pressed: *pressed,
                    pressure: *pressure,
                }),
                ToucheData::TouchFrame {
                    x,
                    y,
                    touch_id,
                    pressed,
                } => Some(ToucheEvent::Touch {
                    x: *x,
                    y: *y,
                    touch_id: *touch_id,
                    pressed: *pressed,
                }),
                ToucheData::ButtonFrame { button_id, pressed } => Some(ToucheEvent::Button {
                    button_id: *button_id,
                    pressed: *pressed,
                }),
                ToucheData::Action(action) => match action {
                    Action::Init => {
                        state = DriverState::Uninitialized;

                        None
                    }
                },
            })
            .collect();

        if let DriverState::Initialized {
            ref mut tablet,
            ref mut touchpad,
        } = state
        {
            tablet.emit(&events[..])?;
            touchpad.emit(&events[..])?;
        }
    }
}
