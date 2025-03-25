mod driver;
mod util;

use std::time::Duration;

use driver::driver_loop;
use futures_lite::stream;
use nusb::{hotplug::HotplugEvent, watch_devices};
use util::aoa::{get_aoa_version, introduce_host, is_aoa, make_aoa};

fn main() {
    for event in stream::block_on(watch_devices().unwrap()) {
        println!("new USB device connected");
        if let HotplugEvent::Connected(device_info) = event {
            std::thread::sleep(Duration::from_millis(100));

            println!("debug: {}", device_info.product_id());

            if is_aoa(&device_info) {
                println!("happy! aoa is ready!");
                driver_loop(device_info).unwrap();
            } else {
                println!("making aoa...");
                if let Ok(handle) = device_info.open() {
                    std::thread::sleep(Duration::from_millis(1000));
                    if handle.claim_interface(0).is_ok() {
                        // AOA stage 1 - determine AOA version
                        let data_stage_1 = get_aoa_version(&handle).unwrap_or_default();
                        println!("getting aoa version");
                        if !data_stage_1.first().is_some_and(|it| (1..=2).contains(it))
                        /* require AOA v1+ */
                        {
                            continue;
                        }
                        // AOA stage 2 - introduce the driver to the Android device
                        println!("introducing");

                        let manufacturer_name = "bpavuk";
                        let model_name = "touche";
                        let description = "making your phone a touchepad and graphics tablet";
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
                    } else {
                        println!("failed to claim the interface");
                        continue;
                    }
                } else {
                    println!("failed to open the device");
                    continue;
                }
            }
        }
    }
}
