pub(crate) mod utils;

use futures_lite::stream;
use log::{debug, error, info};
use nusb::{
    DeviceInfo, Interface, MaybeFuture,
    hotplug::HotplugEvent,
    transfer::{Bulk, Direction, In, Out},
    watch_devices,
};
use std::{
    error::Error,
    io::{self, Read, Write},
    time::Duration,
};
use utils::{get_aoa_version, introduce_host, is_aoa, make_aoa};

pub struct AoaDevice {
    interface: Interface,
    in_endpoint_address: u8,
    out_endpoint_address: u8,
}

impl AoaDevice {
    pub fn new(aoa_device_info: DeviceInfo) -> Result<AoaDevice, Box<dyn Error>> {
        info!("attempting to open the AOA device...");
        let device = aoa_device_info.open().wait().map_err(|_| {
            error!("failed to open the AOA device!");

            io::Error::other("failed to open the AOA device!")
        })?;
        info!("attempting to claim the interface...");
        let interface = device.claim_interface(0).wait().map_err(|e| {
            error!("failed to claim the interface!");
            dbg!(e);

            io::Error::other("failed to claim the interface!")
        })?;

        let binding = interface.clone();
        let descriptors: Vec<_> = binding.descriptors().collect();

        let endpoints: Vec<_> = descriptors
            .iter()
            .flat_map(|desc| desc.endpoints())
            .collect();

        let in_endpoint = endpoints
            .iter()
            .find(|end| end.direction() == Direction::In);
        let in_endpoint = match in_endpoint {
            Some(endpoint) => endpoint,
            None => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::NotFound,
                    "No in endpoints found.",
                )));
            }
        };
        let out_endpoint = endpoints
            .iter()
            .find(|end| end.direction() == Direction::Out);
        let out_endpoint = match out_endpoint {
            Some(endpoint) => endpoint,
            None => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::NotFound,
                    "No out endpoints found.",
                )));
            }
        };
        Ok(AoaDevice {
            interface,
            in_endpoint_address: in_endpoint.address(),
            out_endpoint_address: out_endpoint.address(),
        })
    }

    pub fn read(&self) -> Result<Vec<u8>, std::io::Error> {
        info!("reading...");
        let mut buf = Vec::new();
        // let timeout = Duration::new(1, 0);
        let mut reader = self
            .interface
            .endpoint::<Bulk, In>(self.in_endpoint_address)?
            .reader(256)
            .with_num_transfers(1);

         let mut reader_pkt = reader.until_short_packet();
        // .with_read_timeout(timeout);

        reader_pkt.read_to_end(&mut buf)?;
        reader_pkt.consume_end().map_err(|_| {
            std::io::Error::new(io::ErrorKind::InvalidData, "expected short packet")
        })?;

        info!("done reading");

        Ok(buf)
    }

    pub fn write(&self, data: Vec<u8>) -> Result<(), std::io::Error> {
        info!("writing...");

        let mut writer = self
            .interface
            .endpoint::<Bulk, Out>(self.out_endpoint_address)?
            .writer(16834);

        writer.write_all(&data)?;

        Ok(())
    }
}

pub fn usb_device_listener<T>(callback: T)
where
    T: Fn(AoaDevice),
{
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
                callback(aoa_device);
            } else {
                info!("searching for Android device...");
                if let Ok(device) = device_info.open().wait() {
                    std::thread::sleep(Duration::from_millis(500));

                    #[cfg(target_os = "windows")]
                    let handle = device.claim_interface(0);
                    #[cfg(target_os = "windows")]
                    if handle.is_err() {
                        continue;
                    }
                    #[cfg(target_os = "windows")]
                    let handle = handle.unwrap();

                    #[cfg(target_os = "linux")]
                    let handle = device;

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
                    #[cfg(windows)]
                    let _ = device.reset();
                    #[cfg(unix)]
                    let _ = handle.reset();
                } else {
                    error!("failed to open the device");
                    continue;
                }
            }
        }
    }
}
