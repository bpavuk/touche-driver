use core::result::Result;
use std::time::Duration;

use crate::{
    aoa::AoaDevice,
    data::{ToucheData, parse_touche_data},
    graphics_tablet::GraphicsTabletDevice,
    touchpad::TouchpadDevice,
};

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
        let mut touchetab = match GraphicsTabletDevice::new(*width, *height) {
            Ok(tab) => tab,
            Err(e) => {
                error!("graphics tablet creation error! {}", e);
                info!("error logs:\n{}", e);
                return Err(());
            }
        };
        let mut touchepad = match TouchpadDevice::new(*width, *height) {
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

                    // new approach
                    match touchepad.emit(&events[..]) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("touchpad event processing error!");
                            info!("error logs:\n{}", e);
                            return Err(());
                        }
                    }

                    // graphics tablet events emission
                    match touchetab.emit(&events[..]) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("graphics tablet event processing error!");
                            info!("error logs:\n{}", e);
                        }
                    }

                    trace!("finished parsing data frame");
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(e) => {
                    error!("TRANSFER ERROR! perhaps, device disconnected?");
                    info!("error logs:\n{}", e);
                    break;
                }
            }
        }
    } else {
        error!("wrong size data!");
        info!("error logs:\nWrong size data received from device");
        return Result::Err(());
    }
    Result::Err(())
}
