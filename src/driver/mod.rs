mod device_builder;
mod parser;

use std::time::Duration;

use core::result::Result;
use device_builder::{graphic_tablet, touchpad};
use evdev::InputEvent;
use evdev::{AbsoluteAxisType, EventType, Key};
use futures_lite::{future::block_on, io};
use nusb::{
    transfer::{Direction, RequestBuffer},
    DeviceInfo,
};
use parser::parse;

struct Finger {
    tracking_id: i32,
}

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
                println!("awaiting first touch");
                let buf = RequestBuffer::new(16384);
                let res = block_on(interface.bulk_in(endpoint.address(), buf)).into_result();

                match res {
                    Ok(res) => {
                        println!(
                            "{}",
                            String::from_utf8(res.clone())
                                .unwrap_or("ENCODING FAILURE".to_string())
                        );
                        let res = parse(&res);
                        let header = res.first().unwrap();
                        let height = header[1].parse::<i32>().unwrap();
                        let width = header[0].parse::<i32>().unwrap();

                        let mut touchetab = graphic_tablet(width, height)?;
                        let mut touchepad = touchpad(width, height)?;

                        std::thread::sleep(Duration::from_millis(1000));

                        loop {
                            let buf = RequestBuffer::new(16384);
                            println!("performing request");
                            let res =
                                block_on(interface.bulk_in(endpoint.address(), buf)).into_result();
                            match res {
                                Ok(res) => {
                                    println!(
                                        "{}",
                                        String::from_utf8(res.clone())
                                            .unwrap_or("ENCODING FAILURE".to_string())
                                    );
                                    let info_string = String::from_utf8(res).unwrap();
                                    let events: Vec<Vec<&str>> = info_string
                                        .split("\n")
                                        .map(|it| it.split("\t").collect::<Vec<&str>>())
                                        .collect();
                                    let input_type = events[1][0];

                                    match input_type {
                                        // using stylus events
                                        "S" => {
                                            let x = events[1][1].parse::<f32>().unwrap();
                                            let y = events[1][2].parse::<f32>().unwrap();

                                            let pressed = events[1][3].parse::<i32>().unwrap();

                                            let mut tablet_events: Vec<InputEvent> = vec![
                                                InputEvent::new(
                                                    EventType::KEY,
                                                    Key::BTN_TOOL_PEN.0,
                                                    1,
                                                ),
                                                InputEvent::new(
                                                    EventType::ABSOLUTE,
                                                    AbsoluteAxisType::ABS_X.0,
                                                    x as i32,
                                                ),
                                                InputEvent::new(
                                                    EventType::ABSOLUTE,
                                                    AbsoluteAxisType::ABS_Y.0,
                                                    y as i32,
                                                ),
                                                InputEvent::new(
                                                    EventType::KEY,
                                                    Key::BTN_TOUCH.0,
                                                    pressed,
                                                ),
                                            ];
                                            let pressure = if pressed == 1 {
                                                events[1][4].parse::<f32>().unwrap_or(0_f32)
                                                    * 4096_f32
                                            } else {
                                                0_f32
                                            };
                                            tablet_events.push(InputEvent::new(
                                                EventType::ABSOLUTE,
                                                AbsoluteAxisType::ABS_PRESSURE.0,
                                                pressure as i32,
                                            ));

                                            touchetab.emit(&tablet_events)?;
                                        }
                                        // using touch events
                                        "F" => {
                                            let mut trackpad_events: Vec<InputEvent> = vec![];
                                            let mut fingers: Vec<Finger> = vec![];
                                            for event in &events[1..] {
                                                let x = event[1].parse::<f32>().unwrap();
                                                let y = event[2].parse::<f32>().unwrap();
                                                let pressed = event[3].parse::<i32>().unwrap();
                                                let tracking_id = event[4].parse::<i64>().unwrap();
                                                trackpad_events.append(&mut vec![
                                                    InputEvent::new(
                                                        EventType::ABSOLUTE,
                                                        AbsoluteAxisType::ABS_MT_SLOT.0,
                                                        tracking_id as i32,
                                                    ),
                                                    InputEvent::new(
                                                        EventType::ABSOLUTE,
                                                        AbsoluteAxisType::ABS_MT_TRACKING_ID.0,
                                                        if pressed == 1 {
                                                            tracking_id as i32
                                                        } else {
                                                            -1
                                                        },
                                                    ),
                                                    InputEvent::new(
                                                        EventType::ABSOLUTE,
                                                        AbsoluteAxisType::ABS_MT_POSITION_X.0,
                                                        x as i32,
                                                    ),
                                                    InputEvent::new(
                                                        EventType::ABSOLUTE,
                                                        AbsoluteAxisType::ABS_MT_POSITION_Y.0,
                                                        y as i32,
                                                    ),
                                                    //InputEvent::new(
                                                    //    EventType::KEY,
                                                    //    Key::BTN_TOUCH.0,
                                                    //    pressed,
                                                    //),
                                                ]);
                                                if pressed == 0 {
                                                    let idx = fingers.iter().position(|finger| {
                                                        i64::from(finger.tracking_id) == tracking_id
                                                    });

                                                    if let Some(idx) = idx {
                                                        fingers.remove(idx);
                                                    };
                                                } else if !fingers.iter().any(|finger| {
                                                    i64::from(finger.tracking_id) == tracking_id
                                                }) {
                                                    fingers.push(Finger {
                                                        tracking_id: tracking_id as i32,
                                                    });
                                                }
                                                //if events.last().is_some_and(|it| it != event) {
                                                //    trackpad_events.push(InputEvent::new(
                                                //        EventType::SYNCHRONIZATION,
                                                //        Synchronization::SYN_MT_REPORT.0,
                                                //        1,
                                                //    ));
                                                //}
                                            }

                                            match fingers.len() {
                                                1 => {
                                                    trackpad_events.append(&mut vec![
                                                        InputEvent::new(
                                                            EventType::KEY,
                                                            Key::BTN_TOUCH.0,
                                                            1,
                                                        ),
                                                        InputEvent::new(
                                                            EventType::KEY,
                                                            Key::BTN_TOOL_FINGER.0,
                                                            1,
                                                        ),
                                                        //InputEvent::new(
                                                        //    EventType::KEY,
                                                        //    Key::BTN_TOOL_DOUBLETAP.0,
                                                        //    0,
                                                        //),
                                                    ]);
                                                    println!("performing click!");
                                                }
                                                2 => {
                                                    trackpad_events.append(&mut vec![
                                                        //InputEvent::new(
                                                        //    EventType::KEY,
                                                        //    Key::BTN_TOUCH.0,
                                                        //    1,
                                                        //),
                                                        InputEvent::new(
                                                            EventType::KEY,
                                                            Key::BTN_TOOL_FINGER.0,
                                                            0,
                                                        ),
                                                        InputEvent::new(
                                                            EventType::KEY,
                                                            Key::BTN_TOOL_DOUBLETAP.0,
                                                            1,
                                                        ),
                                                    ]);
                                                    println!("performing rmb!");
                                                }
                                                3 => {}
                                                4 => {}
                                                5 => {}
                                                _ => {
                                                    trackpad_events.append(&mut vec![
                                                        InputEvent::new(
                                                            EventType::KEY,
                                                            Key::BTN_TOUCH.0,
                                                            0,
                                                        ),
                                                        InputEvent::new(
                                                            EventType::KEY,
                                                            Key::BTN_TOOL_FINGER.0,
                                                            0,
                                                        ),
                                                        InputEvent::new(
                                                            EventType::KEY,
                                                            Key::BTN_TOOL_DOUBLETAP.0,
                                                            0,
                                                        ),
                                                    ]);
                                                }
                                            }

                                            println!("finger count: {}", fingers.len());

                                            touchepad.emit(&trackpad_events)?;
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
                    Err(_) => {
                        println!("yoink! disconnect!");
                    }
                }
            }
        }
    }

    Result::Ok(())
}
