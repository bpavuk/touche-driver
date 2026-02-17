use std::io::Write;

use chrono::Utc;
use log::{error, info};
use touche_lib::aoa::source::AoaSource;
use touche_lib::devices::graphics_tablet::GraphicsTabletDevice;
use touche_lib::devices::touchpad::TouchpadDevice;
use touche_lib::devices::DeviceSink;
use touche_lib::driver::Driver;
use touche_lib::touche_aoa::{usb_device_listener, AoaDevice};

fn main() {
    let _ = env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} | {} | {} : {}",
                Utc::now().format("%Y-%m-%d %H:%M:%S.%3f"),
                record.level(),
                record.module_path().unwrap_or("NO_MODULE"),
                record.args()
            )
        })
        .try_init();
    let result = usb_device_listener(|aoa_device_result| {
        let aoa_device: AoaDevice = match aoa_device_result {
            Ok(dev) => dev,
            Err(e) => {
                error!("{}", e);
                return;
            },
        };

        info!("AOA device detected. Starting the driver...");

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

    error!("the driver exited unnaturally!");
    if let Err(e) = result {
        error!("{}", e);
    }
}
