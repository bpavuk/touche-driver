use futures_lite::future::block_on;
use nusb::{
    Device, DeviceInfo,
    transfer::{ControlIn, ControlOut, ControlType, Recipient, ResponseBuffer, TransferError},
};

const MANUFACTURER_NAME_ID: u16 = 0x00;
const MODEL_NAME_ID: u16 = 0x01;
const DESCRIPTION_ID: u16 = 0x02;
const VERSION_ID: u16 = 0x03;
const URI_ID: u16 = 0x04;
const SERIAL_NUMBER_ID: u16 = 0x05;

pub(crate) fn get_aoa_version(handle: &Device) -> Result<Vec<u8>, TransferError> {
    let request = ControlIn {
        control_type: ControlType::Vendor,
        recipient: Recipient::Device,
        request: 51,
        value: 0,
        index: 0,
        length: 16,
    };
    block_on(handle.control_in(request)).into_result()
}

fn send_str(handle: &Device, string: &str, idx: u16) -> Result<ResponseBuffer, TransferError> {
    let request = ControlOut {
        control_type: ControlType::Vendor,
        recipient: Recipient::Device,
        request: 52,
        value: 0,
        data: string.as_bytes(),
        index: idx,
    };

    block_on(handle.control_out(request)).into_result()
}

pub(crate) fn introduce_host(
    handle: &Device,
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

pub(crate) fn make_aoa(handle: &Device) -> Result<ResponseBuffer, TransferError> {
    let request = ControlOut {
        control_type: ControlType::Vendor,
        recipient: Recipient::Device,
        request: 53,
        value: 0,
        index: 0,
        data: &[],
    };

    block_on(handle.control_out(request)).into_result()
}

pub(crate) fn is_aoa(info: &DeviceInfo) -> bool {
    (0x2d00..=0x2d05).contains(&info.product_id())
}
