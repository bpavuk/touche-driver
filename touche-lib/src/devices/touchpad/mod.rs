use std::io::{self, Error};

use crate::{data::events::ToucheEvent, devices::DeviceSink};

#[cfg(target_os = "linux")]
use evdev::{
    AbsInfo, AbsoluteAxisCode, AbsoluteAxisEvent, AttributeSet, BusType, InputEvent, InputId,
    KeyCode, KeyEvent, PropType, UinputAbsSetup, uinput::VirtualDevice,
};
use log::trace;

#[cfg(target_os = "linux")]
pub struct TouchpadDevice {
    device: Option<VirtualDevice>,
}

#[cfg(target_os = "linux")]
impl TouchpadDevice {
    pub fn new_uninit() -> TouchpadDevice {
        TouchpadDevice { device: None }
    }
}

impl DeviceSink for TouchpadDevice {
    fn emit(&mut self, touche_data: &[ToucheEvent]) -> Result<(), io::Error> {
        if self.device.is_none() {
            use std::io::Error;

            return Err(Error::other(
                "device is uninitialized. initialize the device first.",
            ));
        }

        let mut trackpad_events: Vec<InputEvent> = vec![];
        let mut finger_count = 0;
        for event in touche_data {
            match event {
                ToucheEvent::Stylus { .. } => {}
                ToucheEvent::Button { .. } => {}
                ToucheEvent::Touch {
                    x,
                    y,
                    touch_id,
                    pressed,
                } => {
                    trace!("parsing touch frame");
                    let mt_slot = touch_id % 10;
                    if *pressed {
                        finger_count += 1;
                    }

                    trackpad_events.append(&mut vec![
                        *AbsoluteAxisEvent::new(AbsoluteAxisCode::ABS_MT_SLOT, mt_slot),
                        *AbsoluteAxisEvent::new(
                            AbsoluteAxisCode::ABS_MT_TRACKING_ID,
                            if *pressed { *touch_id } else { -1 },
                        ),
                        *AbsoluteAxisEvent::new(AbsoluteAxisCode::ABS_MT_POSITION_X, *x),
                        *AbsoluteAxisEvent::new(AbsoluteAxisCode::ABS_MT_POSITION_Y, *y),
                    ]);
                }
            }
        }

        if !trackpad_events.is_empty() {
            trace!("emitting trackpad events");
            trackpad_events.append(&mut vec![
                *KeyEvent::new(KeyCode::BTN_TOUCH, (1..=5).contains(&finger_count).into()),
                *KeyEvent::new(KeyCode::BTN_TOOL_FINGER, (finger_count == 1).into()),
                *KeyEvent::new(KeyCode::BTN_TOOL_DOUBLETAP, (finger_count == 2).into()),
                *KeyEvent::new(KeyCode::BTN_TOOL_TRIPLETAP, (finger_count == 3).into()),
                *KeyEvent::new(KeyCode::BTN_TOOL_QUADTAP, (finger_count == 4).into()),
                *KeyEvent::new(KeyCode::BTN_TOOL_QUINTTAP, (finger_count == 5).into()),
            ]);
            if let Some(device) = self.device.as_mut() {
                return device.emit(&trackpad_events);
            } else {
                return Err(Error::other(
                    "device is uninitialized. initialize the device first.",
                ));
            }
        }
        Result::Ok(())
    }

    fn init(&mut self, width: i32, height: i32) -> Result<(), Box<dyn std::error::Error>> {
        let mut touchepad_keys: AttributeSet<KeyCode> = AttributeSet::new();
        touchepad_keys.insert(KeyCode::BTN_TOUCH);
        touchepad_keys.insert(KeyCode::BTN_TOOL_FINGER);
        touchepad_keys.insert(KeyCode::BTN_TOOL_DOUBLETAP);
        touchepad_keys.insert(KeyCode::BTN_TOOL_TRIPLETAP);
        touchepad_keys.insert(KeyCode::BTN_TOOL_QUADTAP);
        touchepad_keys.insert(KeyCode::BTN_TOOL_QUINTTAP);

        let mut touchepad_props: AttributeSet<PropType> = AttributeSet::new();
        touchepad_props.insert(PropType::POINTER);

        let device = evdev::uinput::VirtualDevice::builder()?
            .name("touchepad")
            .with_properties(&touchepad_props)?
            .with_keys(&touchepad_keys)?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_MT_SLOT,
                AbsInfo::new(0, 0, 10, 8, 0, 50),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_MT_TRACKING_ID,
                AbsInfo::new(0, 0, 65535, 8, 0, 50),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_MT_POSITION_X,
                AbsInfo::new(0, 0, width, 8, 0, 50),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_MT_POSITION_Y,
                AbsInfo::new(0, 0, height, 8, 0, 50),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_X,
                AbsInfo::new(0, 0, width, 8, 0, 50),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_Y,
                AbsInfo::new(0, 0, height, 8, 0, 50),
            ))?
            .input_id(InputId::new(BusType::BUS_USB, 0x5120, 0x0002, 0x1))
            .build()?;

        self.device = Some(device);

        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub(crate) struct TouchpadDevice {}

#[cfg(target_os = "windows")]
impl TouchpadDevice {
    pub(crate) fn new(width: i32, height: i32) -> Result<TouchpadDevice, Box<dyn Error>> {
        Ok(TouchpadDevice {})
    }

    pub(crate) fn emit(&self, touche_data: &[ToucheEvent]) -> Result<(), Box<dyn Error>> {
        // TODO: implement touchpad emulation on Windows

        Ok(())
    }
}
