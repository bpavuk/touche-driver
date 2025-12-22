use std::io::Write;

use touche_lib::aoa::usb_device_listener;
use chrono::Utc;
use touche_lib::driver::driver_loop;
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
            Err(e) => {
                info!("if at first you don't succeed, die, die again!\n");

                log::error!("{}\n", e);

                log::error!("restarting.");
            }
        };
    });
}
