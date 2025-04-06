pub(crate) mod utils;

use futures_lite::future::block_on;
use log::{error, info};
use nusb::{
    DeviceInfo, Interface,
    transfer::{Direction, RequestBuffer, ResponseBuffer, TransferError},
};

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
