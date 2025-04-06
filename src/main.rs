mod aoa;
mod data;
mod driver;
mod graphics_tablet;
mod touchpad;

use std::{io::Write, time::Duration};

use aoa::{
    AoaDevice,
    utils::{get_aoa_version, introduce_host, is_aoa, make_aoa},
};
use chrono::Utc;
use driver::driver_loop;
use futures_lite::stream;
use log::{debug, error, info};
use nusb::{hotplug::HotplugEvent, watch_devices};

fn main() {
    let _ = env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "{}|{}|{}: {}",
                Utc::now().format("%Y-%m-%d %H:%M:%S.%3f"),
                record.module_path().unwrap_or("NO_MODULE"),
                record.level(),
                record.args()
            )
        })
        .try_init();
    for event in stream::block_on(watch_devices().unwrap()) {
        info!("new USB device connected");
        if let HotplugEvent::Connected(device_info) = event {
            std::thread::sleep(Duration::from_millis(100));

            debug!("connected device product_id: {}", device_info.product_id());

            if is_aoa(&device_info) {
                let aoa_device = match AoaDevice::new(device_info) {
                    Ok(device) => device,
                    Err(_) => {
                        error!("failed to create AOA device!");
                        continue;
                    }
                };
                info!("AOA device detected. starting driver loop...");
                match driver_loop(aoa_device) {
                    Ok(_) => {}
                    Err(_) => {
                        info!("if at first you don't succeed, die, die again!");
                    }
                };
            } else {
                info!("searching for Android device...");
                if let Ok(handle) = device_info.open() {
                    std::thread::sleep(Duration::from_millis(1000));
                    // TODO: make it claim the interface
                    // outside Unix platforms

                    // AOA stage 1 - determine AOA version
                    let data_stage_1 = get_aoa_version(&handle).unwrap_or_default();
                    info!("getting AOA version");
                    if !data_stage_1.first().is_some_and(|it| (1..=2).contains(it))
                    /* require AOA v1+ */
                    {
                        continue;
                    }
                    // AOA stage 2 - introduce the driver to the Android device
                    info!("introducing the driver");

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
                    info!("actually building the AOA device");
                    let _ = make_aoa(&handle);
                    let _ = handle.reset();
                } else {
                    error!("failed to open the device");
                    continue;
                }
            }
        }
    }
}
