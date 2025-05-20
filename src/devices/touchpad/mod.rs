use std::io;

use crate::data::ToucheData;

#[cfg(target_os = "linux")]
use evdev::{
    AbsInfo, AbsoluteAxisCode, AbsoluteAxisEvent, AttributeSet, BusType, InputEvent, InputId,
    KeyCode, KeyEvent, PropType, UinputAbsSetup, uinput::VirtualDevice,
};
use log::trace;

#[cfg(target_os = "linux")]
pub(crate) struct TouchpadDevice {
    device: VirtualDevice,
}

#[cfg(target_os = "linux")]
impl TouchpadDevice {
    pub(crate) fn new(width: i32, height: i32) -> io::Result<TouchpadDevice> {
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
                AbsInfo::new(0, 0, 10, 0, 0, 100),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_MT_TRACKING_ID,
                AbsInfo::new(0, 0, 65535, 0, 0, 100),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_MT_POSITION_X,
                AbsInfo::new(0, 0, width, 0, 0, 100),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_MT_POSITION_Y,
                AbsInfo::new(0, 0, height, 0, 0, 100),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_X,
                AbsInfo::new(0, 0, width, 0, 0, 100),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_Y,
                AbsInfo::new(0, 0, height, 0, 0, 100),
            ))?
            .input_id(InputId::new(BusType::BUS_USB, 0x5120, 0x0002, 0x1))
            .build()?;
        Ok(TouchpadDevice { device })
    }

    pub(crate) fn emit(&mut self, touche_data: &[ToucheData]) -> Result<(), io::Error> {
        let mut trackpad_events: Vec<InputEvent> = vec![];
        let mut finger_count = 0;
        for event in touche_data {
            match event {
                ToucheData::ScreenSize { .. } => {
                    // screen size event - do nothing
                }
                ToucheData::StylusFrame { .. } => {}
                ToucheData::ButtonFrame { .. } => {}
                ToucheData::TouchFrame {
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
            return self.device.emit(&trackpad_events);
        }
        Result::Ok(())
    }
}
