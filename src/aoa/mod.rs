pub(crate) mod utils;

use futures_lite::{future::block_on, stream};
use log::{debug, error, info};
use nusb::{
    DeviceInfo, Interface,
    hotplug::HotplugEvent,
    transfer::{Direction, RequestBuffer, ResponseBuffer, TransferError},
    watch_devices,
};
use utils::{get_aoa_version, introduce_host, is_aoa, make_aoa};
use std::time::Duration;

pub(crate) struct AoaDevice {
    interface: Interface,
    in_endpoint_address: u8,
    out_endpoint_address: u8,
}

impl AoaDevice {
    pub(crate) fn new(aoa_device_info: DeviceInfo) -> Result<AoaDevice, ()> {
        info!("attempting to open the AOA device...");
        let device = aoa_device_info.open().map_err(|_| {
            error!("failed to open the AOA device!");
        })?;
        info!("attempting to claim the interface...");
        let interface = device.claim_interface(0).map_err(|_| {
            error!("failed to claim the interface!");
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
                return Err(());
            }
        };
        let out_endpoint = endpoints
            .iter()
            .find(|end| end.direction() == Direction::Out);
        let out_endpoint = match out_endpoint {
            Some(endpoint) => endpoint,
            None => {
                return Err(());
            }
        };
        Ok(AoaDevice {
            interface,
            in_endpoint_address: in_endpoint.address(),
            out_endpoint_address: out_endpoint.address(),
        })
    }

    pub(crate) fn read(&self) -> Result<Vec<u8>, TransferError> {
        let buffer = RequestBuffer::new(16384);
        block_on(self.interface.bulk_in(self.in_endpoint_address, buffer)).into_result()
    }

    pub(crate) fn write(&self, data: Vec<u8>) -> Result<ResponseBuffer, TransferError> {
        block_on(self.interface.bulk_out(self.out_endpoint_address, data)).into_result()
    }
}

pub(crate) fn usb_device_listener<T>(callback: T)
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
                if let Ok(handle) = device_info.open() {
                    std::thread::sleep(Duration::from_millis(500));
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
