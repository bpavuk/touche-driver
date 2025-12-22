use std::io::Write;

use chrono::Utc;
use log::info;
use touche_lib::aoa::source::AoaSource;
use touche_lib::devices::graphics_tablet::GraphicsTabletDevice;
use touche_lib::devices::touchpad::TouchpadDevice;
use touche_lib::devices::{CombinedSink, DeviceSink};
use touche_lib::driver::Driver;
use touche_lib::touche_aoa::usb_device_listener;

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
        info!("AOA device detected. Starting the refactored driver...");

        let aoa_source = AoaSource::new(aoa_device);
        let tablet_sink = GraphicsTabletDevice::new_uninit();
        let touchpad_sink = TouchpadDevice::new_uninit();

        let device_sink = tablet_sink.combine(touchpad_sink);

        let mut driver = Driver::new(device_sink, aoa_source);

        match driver.start_loop() {
            Ok(_) => {}
            Err(e) => {
                info!("if at first you don't succeed, die, die again!\n");

                log::error!("{}\n", e);

                log::error!("restarting.");
            }
        }
    });
}
