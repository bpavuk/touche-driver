mod devices;

use core::result::Result;
use std::time::Duration;

use crate::{
    aoa::AoaDevice,
    data::{ToucheData, parse_touche_data},
};

use devices::{graphic_tablet, touchpad};
use evdev::{AbsoluteAxisCode, AbsoluteAxisEvent, InputEvent, KeyCode, KeyEvent};
use log::{error, info, trace};
use nusb::DeviceInfo;

// This function didn't hear about single responsibility principle
pub(crate) fn driver_loop(aoa_info: DeviceInfo) -> Result<(), ()> {
    let aoa_device = AoaDevice::new(aoa_info.clone())?;

    let opcode = vec![2];
    match aoa_device.write(opcode) {
        Ok(_) => {}
        Err(e) => {
            error!("opcode writing error! {}", e);
            info!("error logs:\n{}", e);
            return Err(());
        }
    }

    let size_data_raw = match aoa_device.read() {
        Ok(data) => data,
        Err(e) => {
            error!("size data retrieval error! maybe, disconnected? {}", e);
            info!("error logs:\n{}", e);
            return Err(());
        }
    };

    let size_data = match parse_touche_data(&size_data_raw) {
        Ok(data) => data,
        Err(e) => {
            error!("data decoding error!");
            info!("error logs:\n{}", e);
            return Err(());
        }
    };

    if let Some(ToucheData::ScreenSize {
        x: width,
        y: height,
    }) = size_data.first()
    {
        let touchetab = graphic_tablet(*width, *height);
        let mut touchetab = match touchetab {
            Ok(tab) => tab,
            Err(e) => {
                error!("graphics tablet creation error! {}", e);
                info!("error logs:\n{}", e);
                return Err(());
            }
        };
        let touchepad = touchpad(*width, *height);
        let mut touchepad = match touchepad {
            Ok(pad) => pad,
            Err(e) => {
                error!("touchpad creation error! {}", e);
                info!("error logs:\n{}", e);
                return Err(());
            }
        };

        std::thread::sleep(Duration::from_millis(30));

        loop {
            trace!("requesting data frame");
            let opcode = vec![1];
            let write_result = aoa_device.write(opcode);
            match write_result {
                Ok(_) => {}
                Err(e) => {
                    error!("opcode writing error! {}", e);
                    info!("error logs:\n{}", e);
                    return Err(());
                }
            }

            let res = aoa_device.read();

            trace!("received. parsing data frame...");
            match res {
                Ok(res) => {
                    let events = parse_touche_data(&res);
                    let events = match events {
                        Ok(events) => events,
                        Err(e) => {
                            error!("data decoding error!");
                            info!("error logs:\n{}", e);
                            return Err(());
                        }
                    };

                    let mut tablet_events: Vec<InputEvent> = vec![];
                    let mut trackpad_events: Vec<InputEvent> = vec![];
                    let mut finger_count = 0;
                    for event in events {
                        match event {
                            ToucheData::ScreenSize { .. } => {
                                // screen size event - do nothing
                            }
                            ToucheData::StylusFrame {
                                x,
                                y,
                                pressed,
                                pressure,
                            } => {
                                trace!("parsing stylus frame");
                                tablet_events.push(*KeyEvent::new(KeyCode::BTN_TOOL_PEN, 1));
                                tablet_events
                                    .push(*AbsoluteAxisEvent::new(AbsoluteAxisCode::ABS_X, x));
                                tablet_events
                                    .push(*AbsoluteAxisEvent::new(AbsoluteAxisCode::ABS_Y, y));
                                tablet_events
                                    .push(*KeyEvent::new(KeyCode::BTN_TOUCH, pressed.into()));
                                let pressure_value = if pressed {
                                    pressure.unwrap_or(1_f32) * 4096_f32
                                } else {
                                    0_f32
                                };
                                tablet_events.push(*AbsoluteAxisEvent::new(
                                    AbsoluteAxisCode::ABS_PRESSURE,
                                    pressure_value as i32,
                                ));
                            }
                            ToucheData::TouchFrame {
                                x,
                                y,
                                touch_id,
                                pressed,
                            } => {
                                trace!("parsing touch frame");
                                let mt_slot = touch_id % 10;
                                finger_count += 1;

                                trackpad_events.append(&mut vec![
                                    *AbsoluteAxisEvent::new(AbsoluteAxisCode::ABS_MT_SLOT, mt_slot),
                                    *AbsoluteAxisEvent::new(
                                        AbsoluteAxisCode::ABS_MT_TRACKING_ID,
                                        if pressed { touch_id } else { -1 },
                                    ),
                                    *AbsoluteAxisEvent::new(AbsoluteAxisCode::ABS_MT_POSITION_X, x),
                                    *AbsoluteAxisEvent::new(AbsoluteAxisCode::ABS_MT_POSITION_Y, y),
                                ]);
                            }
                        }
                    }

                    if !tablet_events.is_empty() {
                        trace!("emitting tablet events");
                        match touchetab.emit(&tablet_events) {
                            Ok(_) => {}
                            Err(e) => {
                                error!("error emitting tablet events! {}", e);
                                info!("error logs:\n{}", e);
                            }
                        }
                    }
                    if !trackpad_events.is_empty() {
                        trace!("emitting trackpad events");
                        trackpad_events.append(&mut vec![
                            *KeyEvent::new(
                                KeyCode::BTN_TOUCH,
                                (1..=5).contains(&finger_count).into(),
                            ),
                            *KeyEvent::new(KeyCode::BTN_TOOL_FINGER, (finger_count == 1).into()),
                            *KeyEvent::new(KeyCode::BTN_TOOL_DOUBLETAP, (finger_count == 2).into()),
                            *KeyEvent::new(KeyCode::BTN_TOOL_TRIPLETAP, (finger_count == 3).into()),
                            *KeyEvent::new(KeyCode::BTN_TOOL_QUADTAP, (finger_count == 4).into()),
                            *KeyEvent::new(KeyCode::BTN_TOOL_QUINTTAP, (finger_count == 5).into()),
                        ]);
                        match touchepad.emit(&trackpad_events) {
                            Ok(_) => {}
                            Err(e) => {
                                error!("error emitting touchpad events! {}", e);
                                info!("error logs:\n{}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("TRANSFER ERROR! perhaps, device disconnected?");
                    info!("error logs:\n{}", e);
                    break;
                }
            }
            trace!("finished parsing data frame");
        }
    } else {
        error!("wrong size data!");
        info!("error logs:\nWrong size data received from device");
        return Result::Err(());
    }
    Result::Err(())
}
