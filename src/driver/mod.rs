mod devices;

use core::result::Result;
use std::time::Duration;

use crate::data::{ToucheData, parse_touche_data};

use devices::{graphic_tablet, touchpad};
use evdev::{AbsoluteAxisCode, AbsoluteAxisEvent, InputEvent, KeyCode, KeyEvent};
use futures_lite::{future::block_on, io};
use log::{error, info, trace};
use nusb::{
    DeviceInfo,
    transfer::{Direction, RequestBuffer},
};

// This function didn't hear about single responsibility principle
pub(crate) fn driver_loop(aoa_info: DeviceInfo) -> io::Result<()> {
    info!("attempting to open the AOA device...");
    if let Ok(device) = aoa_info.open() {
        info!("attempting to claim the interface...");
        if let Ok(interface) = device.claim_interface(0) {
            info!("interface claimed. searching for IN and OUT endpoints...");
            let descriptors = interface.descriptors().collect::<Vec<_>>();
            let endpoints = descriptors
                .iter()
                .flat_map(|desc| desc.endpoints())
                .collect::<Vec<_>>();

            if let (Some(in_endpoint), Some(out_endpoint)) = (
                endpoints
                    .iter()
                    .find(|end| end.direction() == Direction::In),
                endpoints
                    .iter()
                    .find(|end| end.direction() == Direction::Out),
            ) {
                let opcode = vec![2];
                let _ = block_on(interface.bulk_out(out_endpoint.address(), opcode));

                let size_data_buf = RequestBuffer::new(16384);
                let size_data_res =
                    block_on(interface.bulk_in(in_endpoint.address(), size_data_buf)).into_result();

                if size_data_res.is_err() {
                    error!("size data retrieval error! maybe, disconnected?");
                    return Result::Ok(());
                }

                let size_data = parse_touche_data(&size_data_res.unwrap());
                if size_data.is_err() {
                    error!("data decoding error!");
                    return Result::Ok(());
                }
                let size_data = size_data.unwrap();

                if let Some(ToucheData::ScreenSize {
                    x: width,
                    y: height,
                }) = size_data.first()
                {
                    let mut touchetab = graphic_tablet(*width, *height)?;
                    let mut touchepad = touchpad(*width, *height)?;

                    std::thread::sleep(Duration::from_millis(30));

                    loop {
                        trace!("requesting data frame");
                        let opcode = vec![1];
                        let _ = block_on(interface.bulk_out(out_endpoint.address(), opcode))
                            .into_result();

                        let buf = RequestBuffer::new(16384);
                        let res =
                            block_on(interface.bulk_in(in_endpoint.address(), buf)).into_result();

                        trace!("received. parsing data frame...");
                        match res {
                            Ok(res) => {
                                let events = parse_touche_data(&res);
                                if events.is_err() {
                                    error!("data decoding error!");
                                }
                                let events = events.unwrap();

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
                                            tablet_events
                                                .push(*KeyEvent::new(KeyCode::BTN_TOOL_PEN, 1));
                                            tablet_events.push(*AbsoluteAxisEvent::new(
                                                AbsoluteAxisCode::ABS_X,
                                                x,
                                            ));
                                            tablet_events.push(*AbsoluteAxisEvent::new(
                                                AbsoluteAxisCode::ABS_Y,
                                                y,
                                            ));
                                            tablet_events.push(*KeyEvent::new(
                                                KeyCode::BTN_TOUCH,
                                                pressed.into(),
                                            ));
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
                                            let mt_slot = touch_id % 10;
                                            finger_count += 1;

                                            trackpad_events.append(&mut vec![
                                                *AbsoluteAxisEvent::new(
                                                    AbsoluteAxisCode::ABS_MT_SLOT,
                                                    mt_slot,
                                                ),
                                                *AbsoluteAxisEvent::new(
                                                    AbsoluteAxisCode::ABS_MT_TRACKING_ID,
                                                    if pressed { touch_id } else { -1 },
                                                ),
                                                *AbsoluteAxisEvent::new(
                                                    AbsoluteAxisCode::ABS_MT_POSITION_X,
                                                    x,
                                                ),
                                                *AbsoluteAxisEvent::new(
                                                    AbsoluteAxisCode::ABS_MT_POSITION_Y,
                                                    y,
                                                ),
                                            ]);
                                        }
                                    }
                                }

                                if !tablet_events.is_empty() {
                                    touchetab.emit(&tablet_events)?;
                                }
                                if !trackpad_events.is_empty() {
                                    trackpad_events.append(&mut vec![*KeyEvent::new(
                                        KeyCode::BTN_TOUCH,
                                        (1..=5).contains(&finger_count).into(),
                                    )]);
                                    touchepad.emit(&trackpad_events)?;
                                }
                            }
                            Err(_) => {
                                error!("TRANSFER ERROR! perhaps, device disconnected?");
                                break;
                            }
                        }
                        trace!("finished parsing data frame");
                    }
                } else {
                    error!("wrong size data!");
                    return Result::Ok(());
                }
            }
        }
    }

    Result::Ok(())
}
