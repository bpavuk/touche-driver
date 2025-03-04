mod driver;
mod util;

use std::{time::Duration, vec::Vec};

use driver::driver_loop;
use mio::{Events, Interest, Poll, Token};
use nusb::list_devices;
use udev::MonitorBuilder;
use util::aoa::{check_aoa, get_aoa_version, introduce_host, make_aoa};

fn main() {
    let mut udev_monitor = MonitorBuilder::new()
        .expect("FAILED TO BUILD A DEVICE MONITOR")
        .match_subsystem("usb")
        .expect("FAILED TO SET USB SUBSYSTEM FILTER")
        .listen()
        .expect("FAILED TO START LISTENING TO THE MONITOR");

    let mut poll = Poll::new().expect("FAILED TO BUILD A POLL");
    let mut events = Events::with_capacity(2048);

    poll.registry()
        .register(&mut udev_monitor, Token(0), Interest::READABLE)
        .expect("FAILED TO REGISTER THE MONITOR");

    loop {
        std::thread::sleep(Duration::from_millis(100));
        let devices = list_devices();
        if devices.is_err() {
            continue;
        };
        let devices = devices.unwrap();

        let maybe_aoa = check_aoa();
        match maybe_aoa {
            Some(device_info) => {
                println!("happy! aoa is ready!");
                driver_loop(device_info).unwrap();
            }
            None => {
                println!("making aoa...");
                let binding: Vec<nusb::DeviceInfo> = devices
                    .filter(|dev| {
                        if let Ok(handle) = dev.open() {
                            if handle.claim_interface(0).is_ok() {
                                // AOA stage 1 - determine AOA version
                                let data_stage_1 = get_aoa_version(&handle).unwrap_or_default();
                                println!("getting aoa version");
                                if !data_stage_1.first().is_some_and(|it| (1..=2).contains(it))
                                /* require AOA v1+ */
                                {
                                    return false;
                                }
                                // AOA stage 2 - introduce the driver to the Android device
                                println!("introducing");

                                let manufacturer_name = "bpavuk";
                                let model_name = "touche";
                                let description =
                                    "making your phone a touchepad and graphics tablet";
                                let version = "v0"; // TODO: change to v1 once it's done
                                let uri = "what://"; // TODO
                                let serial_number = "528491"; // have you ever watched Inception?

                                introduce_host(
                                    &handle,
                                    manufacturer_name,
                                    model_name,
                                    description,
                                    version,
                                    uri,
                                    serial_number,
                                );

                                // AOA stage 3 - make Android your accessory
                                println!("actually making");
                                let _ = make_aoa(&handle);
                                let _ = handle.reset();
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    })
                    .collect();

                std::thread::sleep(Duration::from_millis(100));
                if let Some(_android_device) = binding.first() {
                    println!("woah! at least one device is ready");
                    std::thread::sleep(Duration::from_millis(1));

                    let aoa_device_info = check_aoa();

                    if let Some(dev) = aoa_device_info {
                        println!("found the aoa device");
                        std::thread::sleep(Duration::from_millis(1));
                        driver_loop(dev).unwrap();
                    } else {
                        println!("whoops! it isn't ready");
                    }
                } else {
                    println!("no devices were connected");
                }
            }
        }

        poll.poll(&mut events, None).unwrap(); // await for the next device to connect
    }
}
