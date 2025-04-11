use std::io;

use crate::data::ToucheData;

#[cfg(target_os = "linux")]
use evdev::{
    AbsInfo, AbsoluteAxisCode, AbsoluteAxisEvent, AttributeSet, BusType, InputEvent, InputId,
    KeyCode, KeyEvent, PropType, UinputAbsSetup, uinput::VirtualDevice,
};
use log::error;

#[cfg(target_os = "linux")]
pub(crate) struct GraphicsTabletDevice {
    device: VirtualDevice,
}

#[cfg(target_os = "linux")]
impl GraphicsTabletDevice {
    pub(crate) fn new(width: i32, height: i32) -> io::Result<GraphicsTabletDevice> {
        println!("device setup. width {} height {}", width, height);
        let mut touche_tablet_keys: AttributeSet<KeyCode> = AttributeSet::new();

        // defining stylus capabilities
        touche_tablet_keys.insert(KeyCode::BTN_STYLUS);
        touche_tablet_keys.insert(KeyCode::BTN_TOOL_PEN);
        touche_tablet_keys.insert(KeyCode::BTN_TOUCH);

        // defining on-tablet buttons
        touche_tablet_keys.insert(KeyCode::BTN_0);
        touche_tablet_keys.insert(KeyCode::BTN_1);
        touche_tablet_keys.insert(KeyCode::BTN_2);
        touche_tablet_keys.insert(KeyCode::BTN_3);
        touche_tablet_keys.insert(KeyCode::BTN_4);
        touche_tablet_keys.insert(KeyCode::BTN_5);
        touche_tablet_keys.insert(KeyCode::BTN_6);
        touche_tablet_keys.insert(KeyCode::BTN_7);
        touche_tablet_keys.insert(KeyCode::BTN_8);
        touche_tablet_keys.insert(KeyCode::BTN_9);

        // props set accordingly to specifications in Linux kernel docs
        let mut touche_tablet_props: AttributeSet<PropType> = AttributeSet::new();
        touche_tablet_props.insert(PropType::DIRECT);
        touche_tablet_props.insert(PropType::POINTER);

        let device = evdev::uinput::VirtualDevice::builder()?
            .name("touchetab")
            .with_properties(&touche_tablet_props)?
            .with_keys(&touche_tablet_keys)?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_X,
                AbsInfo::new(0, 0, width, 0, 0, 100),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_Y,
                AbsInfo::new(0, 0, height, 0, 0, 100),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_PRESSURE,
                AbsInfo::new(0, 0, 4096, 0, 0, 100),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_DISTANCE,
                AbsInfo::new(0, 0, 1024, 0, 0, 100),
            ))?
            .input_id(InputId::new(BusType::BUS_USB, 0x5120, 0x0001, 0x1))
            .build()?;
        Ok(GraphicsTabletDevice { device })
    }

    pub(crate) fn emit(&mut self, touche_data: &[ToucheData]) -> Result<(), io::Error> {
        let mut tablet_events: Vec<InputEvent> = vec![];
        for event in touche_data {
            match event {
                ToucheData::ScreenSize { .. } => {
                    // screen size event - do nothing
                }
                ToucheData::TouchFrame { .. } => {
                    // touch frame - ignore
                }
                ToucheData::StylusFrame {
                    x,
                    y,
                    pressed,
                    pressure,
                } => {
                    let tool_pen_event = *KeyEvent::new(KeyCode::BTN_TOOL_PEN, 1);
                    let x_event = *AbsoluteAxisEvent::new(AbsoluteAxisCode::ABS_X, *x);
                    let y_event = *AbsoluteAxisEvent::new(AbsoluteAxisCode::ABS_Y, *y);
                    let touch_event = *KeyEvent::new(KeyCode::BTN_TOUCH, (*pressed).into());

                    tablet_events.push(tool_pen_event);
                    tablet_events.push(x_event);
                    tablet_events.push(y_event);
                    tablet_events.push(touch_event);

                    let pressure_int = if let Some(pressure_value) = pressure {
                        (pressure_value * 4096.0) as i32 // Assuming max pressure is 4096
                    } else {
                        0
                    };
                    tablet_events.push(*AbsoluteAxisEvent::new(
                        AbsoluteAxisCode::ABS_PRESSURE,
                        pressure_int,
                    ));
                }
                ToucheData::ButtonFrame { button_id, pressed } => {
                    let key_code = match button_id {
                        0 => KeyCode::BTN_0,
                        1 => KeyCode::BTN_1,
                        2 => KeyCode::BTN_2,
                        3 => KeyCode::BTN_3,
                        4 => KeyCode::BTN_4,
                        5 => KeyCode::BTN_5,
                        6 => KeyCode::BTN_6,
                        7 => KeyCode::BTN_7,
                        8 => KeyCode::BTN_8,
                        9 => KeyCode::BTN_9,
                        _ => {
                            error!("unsupported button id {}", button_id);
                            continue;
                        }
                    };
                    tablet_events.push(*KeyEvent::new(key_code, (*pressed).into()));
                }
            }
        }
        if !tablet_events.is_empty() {
            return self.device.emit(&tablet_events);
        }
        Result::Ok(())
    }
}
