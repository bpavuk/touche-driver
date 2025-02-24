mod devices;

use core::f32;
use core::result::Result;
use std::time::Duration;

use crate::util::parser::parse;

use devices::{graphic_tablet, touchpad};
use evdev::InputEvent;
use evdev::{AbsoluteAxisCode, AbsoluteAxisEvent, KeyCode, KeyEvent};
use futures_lite::{future::block_on, io};
use nusb::{
    DeviceInfo,
    transfer::{Direction, RequestBuffer},
};

pub(crate) fn driver_loop(aoa_info: DeviceInfo) -> io::Result<()> {
    std::thread::sleep(Duration::from_millis(2200));
    println!("attempting to open the device...");
    if let Ok(device) = aoa_info.open() {
        println!("attempting to claim the interface...");
        if let Ok(interface) = device.claim_interface(0) {
            println!("interface claimed. searching for IN endpoint...");
            let descriptors = interface.descriptors().collect::<Vec<_>>();
            let endpoints = descriptors
                .iter()
                .flat_map(|desc| desc.endpoints())
                .collect::<Vec<_>>();

            if let Some(endpoint) = endpoints
                .iter()
                .find(|end| end.direction() == Direction::In)
            {
                let height = 4096;
                let width = 4096;

                let mut touchetab = graphic_tablet(width, height)?;
                let mut touchepad = touchpad(width, height)?;

                std::thread::sleep(Duration::from_millis(30));

                println!("awaiting first touch");
                loop {
                    let buf = RequestBuffer::new(16384);
                    println!("performing request");
                    let res = block_on(interface.bulk_in(endpoint.address(), buf)).into_result();
                    match res {
                        Ok(res) => {
                            println!(
                                "{}",
                                String::from_utf8(res.clone())
                                    .unwrap_or("ENCODING FAILURE".to_string())
                            );
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
                                    println!("finger count: {}", finger_count);
                                    match finger_count {
                                        0 => {
                                            trackpad_events.append(&mut vec![
                                                *KeyEvent::new(KeyCode::BTN_TOUCH, 0),
                                                *KeyEvent::new(KeyCode::BTN_TOOL_FINGER, 0),
                                            ]);
                                        }
                                        1 => {
                                            trackpad_events.append(&mut vec![
                                                *KeyEvent::new(KeyCode::BTN_TOUCH, 1),
                                                *KeyEvent::new(KeyCode::BTN_TOOL_FINGER, 1),
                                                *KeyEvent::new(KeyCode::BTN_TOOL_DOUBLETAP, 0),
                                            ]);
                                        }
                                        2 => {
                                            trackpad_events.append(&mut vec![
                                                *KeyEvent::new(KeyCode::BTN_TOUCH, 1),
                                                *KeyEvent::new(KeyCode::BTN_TOOL_FINGER, 0),
                                                *KeyEvent::new(KeyCode::BTN_TOOL_DOUBLETAP, 1),
                                            ]);
                                        }
                                        3 => {
                                            trackpad_events.append(&mut vec![
                                                *KeyEvent::new(KeyCode::BTN_TOUCH, 1),
                                                *KeyEvent::new(KeyCode::BTN_TOOL_DOUBLETAP, 0),
                                                *KeyEvent::new(KeyCode::BTN_TOOL_TRIPLETAP, 1),
                                            ]);
                                        }
                                        4 => {
                                            trackpad_events.append(&mut vec![
                                                *KeyEvent::new(KeyCode::BTN_TOUCH, 1),
                                                *KeyEvent::new(KeyCode::BTN_TOOL_TRIPLETAP, 0),
                                                *KeyEvent::new(KeyCode::BTN_TOOL_QUADTAP, 1),
                                            ]);
                                        }
                                        5 => {
                                            trackpad_events.append(&mut vec![
                                                *KeyEvent::new(KeyCode::BTN_TOUCH, 1),
                                                *KeyEvent::new(KeyCode::BTN_TOOL_QUADTAP, 0),
                                                *KeyEvent::new(KeyCode::BTN_TOOL_QUINTTAP, 1),
                                            ]);
                                        }
                                        _ => {}
                                    }

                                    touchepad.emit(&trackpad_events)?;

                                    std::thread::sleep(Duration::new(0, 1000));
                                }
                                _ => {}
                            }

                            println!("sent");
                        }
                        Err(_) => {
                            println!("TRANSFER ERROR\nperhaps, device disconnected?");
                            break;
                        }
                    }

                    println!();
                }
            }
        }
    }

    Result::Ok(())
}
