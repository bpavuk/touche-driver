mod devices;

use core::f32;
use core::result::Result;
use std::time::Duration;

use crate::util::parser::parse;

use devices::{graphic_tablet, touchpad};
use evdev::{AbsoluteAxisCode, AbsoluteAxisEvent, InputEvent, KeyCode, KeyEvent};
use futures_lite::{future::block_on, io};
use log::{error, info, trace};
use nusb::{
    DeviceInfo,
    transfer::{Direction, RequestBuffer},
};

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

                let size_data = parse(&size_data_res.unwrap());

                if size_data[0][0] != "X" {
                    error!("wrong size data!");
                    return Result::Ok(());
                }

                let width: i32 = size_data[0][1].parse().unwrap();
                let height: i32 = size_data[0][2].parse().unwrap();

                let mut touchetab = graphic_tablet(width, height)?;
                let mut touchepad = touchpad(width, height)?;

                std::thread::sleep(Duration::from_millis(30));

                loop {
                    trace!("requesting data frame");
                    let opcode = vec![1];
                    let _ =
                        block_on(interface.bulk_out(out_endpoint.address(), opcode)).into_result();

                    let buf = RequestBuffer::new(16384);
                    let res = block_on(interface.bulk_in(in_endpoint.address(), buf)).into_result();

                    trace!("received. parsing data frame...");
                    match res {
                        Ok(res) => {
                            let events: Vec<Vec<String>> = parse(&res);
                            let input_type = events[0][0].as_str();

                            match input_type {
                                // using stylus events
                                "S" => {
                                    let x = events[0][1].parse::<f32>().unwrap();
                                    let y = events[0][2].parse::<f32>().unwrap();

                                    let pressed = events[0][3].parse::<i32>().unwrap();

                                    let mut tablet_events: Vec<InputEvent> = vec![
                                        *KeyEvent::new(KeyCode::BTN_TOOL_PEN, 1),
                                        *AbsoluteAxisEvent::new(AbsoluteAxisCode::ABS_X, x as i32),
                                        *AbsoluteAxisEvent::new(AbsoluteAxisCode::ABS_Y, y as i32),
                                        *KeyEvent::new(KeyCode::BTN_TOUCH, pressed),
                                    ];
                                    let pressure = if pressed == 1 {
                                        events[0][4].parse::<f32>().unwrap_or(0_f32) * 4096_f32
                                    } else {
                                        0_f32
                                    };
                                    tablet_events.push(*AbsoluteAxisEvent::new(
                                        AbsoluteAxisCode::ABS_PRESSURE,
                                        pressure as i32,
                                    ));
                                    touchetab.emit(&tablet_events)?;

                                    std::thread::sleep(Duration::from_millis(5));

                                    trace!("parsed stylus frame");
                                }
                                // using touch events
                                "F" => {
                                    let mut trackpad_events: Vec<InputEvent> = vec![];

                                    for event in &events {
                                        // setting ids, coords
                                        let x = event[1].parse::<f32>().unwrap() as i32;
                                        let y = event[2].parse::<f32>().unwrap() as i32;
                                        let pressed = event[3].parse::<i32>().unwrap() == 1;
                                        let tracking_id = event[4].parse::<i32>().unwrap();
                                        let mt_slot = tracking_id % 10;

                                        trackpad_events.append(&mut vec![
                                            *AbsoluteAxisEvent::new(
                                                AbsoluteAxisCode::ABS_MT_SLOT,
                                                mt_slot,
                                            ),
                                            *AbsoluteAxisEvent::new(
                                                AbsoluteAxisCode::ABS_MT_TRACKING_ID,
                                                if pressed { tracking_id } else { -1 },
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

                                    // invoking tap/doubletap events

                                    let fingers: Vec<_> = events
                                        .iter()
                                        .filter(|event| {
                                            event.get(3).is_some_and(|pressed| {
                                                pressed.parse::<i32>().is_ok_and(|num| num == 1)
                                            })
                                        })
                                        .collect();
                                    let finger_count = fingers.len();
                                    trackpad_events.append(&mut vec![
                                        *KeyEvent::new(
                                            KeyCode::BTN_TOUCH,
                                            (1..=5).contains(&finger_count).into(),
                                        ),
                                        *KeyEvent::new(
                                            KeyCode::BTN_TOOL_FINGER,
                                            (finger_count == 1).into(),
                                        ),
                                        *KeyEvent::new(
                                            KeyCode::BTN_TOOL_DOUBLETAP,
                                            (finger_count == 2).into(),
                                        ),
                                        *KeyEvent::new(
                                            KeyCode::BTN_TOOL_TRIPLETAP,
                                            (finger_count == 3).into(),
                                        ),
                                        *KeyEvent::new(
                                            KeyCode::BTN_TOOL_QUADTAP,
                                            (finger_count == 4).into(),
                                        ),
                                        *KeyEvent::new(
                                            KeyCode::BTN_TOOL_QUINTTAP,
                                            (finger_count == 5).into(),
                                        ),
                                    ]);

                                    touchepad.emit(&trackpad_events)?;

                                    std::thread::sleep(Duration::from_millis(5));

                                    trace!("parsed touchpad frame");
                                }
                                _ => {}
                            }
                        }
                        Err(_) => {
                            error!("TRANSFER ERROR! perhaps, device disconnected?");
                            break;
                        }
                    }
                    trace!("finished parsing data frame");
                }
            }
        }
    }

    Result::Ok(())
}
