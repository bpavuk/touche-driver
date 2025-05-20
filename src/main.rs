mod aoa;
mod data;
mod devices;
mod driver;
use std::io::Write;

use aoa::usb_device_listener;
use chrono::Utc;
use driver::driver_loop;
use log::info;

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
    usb_device_listener(|aoa_device| {
        info!("AOA device detected. starting driver loop...");
        match driver_loop(aoa_device) {
            Ok(_) => {}
            Err(_) => {
                info!("if at first you don't succeed, die, die again!");
            }
        };
    });
}
