use std::error::Error;
use touche_aoa::AoaDevice;

use crate::{
    data::ToucheData,
    devices::ToucheSource,
};

pub struct AoaSource {
    aoa_device: AoaDevice,
}

impl AoaSource {
    pub fn new(device: AoaDevice) -> AoaSource {
        AoaSource { aoa_device: device }
    }
}

impl ToucheSource for AoaSource {
    fn blocking_read(&mut self) -> Result<Vec<ToucheData>, Box<dyn Error>> {
        let raw = self.aoa_device.read()?;
        let data: Vec<ToucheData> = serde_cbor::from_slice(&raw[..])?;

        Ok(data)
    }
}
