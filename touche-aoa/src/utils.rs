use std::time::Duration;

use nusb::{
    Device, DeviceInfo, MaybeFuture,
    transfer::{ControlIn, ControlOut, ControlType, Recipient, TransferError},
};

const MANUFACTURER_NAME_ID: u16 = 0x00;
const MODEL_NAME_ID: u16 = 0x01;
const DESCRIPTION_ID: u16 = 0x02;
const VERSION_ID: u16 = 0x03;
const URI_ID: u16 = 0x04;
const SERIAL_NUMBER_ID: u16 = 0x05;

#[cfg(target_os = "linux")]
type Handle = Device;
#[cfg(target_os = "windows")]
type Handle = Interface;

pub(crate) fn get_aoa_version(handle: &Handle) -> Result<Vec<u8>, TransferError> {
    let request = ControlIn {
        control_type: ControlType::Vendor,
        recipient: Recipient::Device,
        request: 51,
        value: 0,
        index: 0,
        length: 16,
    };
    let timeout = Duration::new(10, 0);
    handle.control_in(request, timeout).wait()
}

fn send_str(handle: &Handle, string: &str, idx: u16) -> Result<(), TransferError> {
    let request = ControlOut {
        control_type: ControlType::Vendor,
        recipient: Recipient::Device,
        request: 52,
        value: 0,
        data: string.as_bytes(),
        index: idx,
    };
    let timeout = Duration::new(10, 0);
    handle.control_out(request, timeout).wait()
}

pub(crate) fn introduce_host(
    handle: &Handle,
    manufacturer_name: &str,
    model_name: &str,
    description: &str,
    version: &str,
    uri: &str,
    serial_number: &str,
) {
    let _ = send_str(handle, manufacturer_name, MANUFACTURER_NAME_ID);
    let _ = send_str(handle, model_name, MODEL_NAME_ID);
    let _ = send_str(handle, description, DESCRIPTION_ID);
    let _ = send_str(handle, version, VERSION_ID);
    let _ = send_str(handle, uri, URI_ID);
    let _ = send_str(handle, serial_number, SERIAL_NUMBER_ID);
}

pub(crate) fn make_aoa(handle: &Handle) -> Result<(), TransferError> {
    let request = ControlOut {
        control_type: ControlType::Vendor,
        recipient: Recipient::Device,
        request: 53,
        value: 0,
        index: 0,
        data: &[],
    };
    let timeout = Duration::new(10, 0);
    handle.control_out(request, timeout).wait()
}

pub(crate) fn is_aoa(info: &DeviceInfo) -> bool {
    (0x2d00..=0x2d05).contains(&info.product_id())
}
