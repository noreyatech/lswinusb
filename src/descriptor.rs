use serde::{Deserialize, Serialize};
use std::fmt;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct UsbDeviceDescriptor {
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub bcdUSB: u16,
    pub bDeviceClass: u8,
    pub bDeviceSubClass: u8,
    pub bDeviceProtocol: u8,
    pub bMaxPacketSize0: u8,
    pub idVendor: u16,
    pub idProduct: u16,
    pub bcdDevice: u16,
    pub iManufacturer: (u8, Option<String>),
    pub iProduct: (u8, Option<String>),
    pub iSerialNumber: (u8, Option<String>),
    pub bNumConfigurations: u8,
}

impl fmt::Debug for UsbDeviceDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut res = "\n".to_string();
        res.push_str(format!("bLength: {}\n", self.bLength).as_str());
        res.push_str(format!("bDescriptorType: {:#04x}\n", self.bDescriptorType).as_str());
        res.push_str(format!("bcdUSB: {:#06x}\n", self.bcdUSB).as_str());
        res.push_str(format!("bDeviceClass: {:#04x}\n", self.bDeviceClass).as_str());
        res.push_str(format!("bDeviceSubClass: {:#04x}\n", self.bDeviceSubClass).as_str());
        res.push_str(format!("bDeviceProtocol: {:#04x}\n", self.bDeviceProtocol).as_str());
        res.push_str(format!("bMaxPacketSize0: {}\n", self.bMaxPacketSize0).as_str());

        res.push_str(format!("idVendor: {:#06x}\n", self.idVendor).as_str());
        res.push_str(format!("idProduct: {:#06x}\n", self.idProduct).as_str());

        res.push_str(format!("bcdDevice: {:#06x}\n", self.bcdDevice).as_str());
        res.push_str(
            format!(
                "iManufacturer: {:#04x} {:?}\n",
                self.iManufacturer.0, self.iManufacturer.1
            )
            .as_str(),
        );
        res.push_str(
            format!("iProduct: {:#04x} {:?}\n", self.iProduct.0, self.iProduct.1).as_str(),
        );
        res.push_str(
            format!(
                "iSerialNumber: {:#04x} {:?}\n",
                self.iSerialNumber.0, self.iSerialNumber.1
            )
            .as_str(),
        );

        res.push_str(format!("bNumConfigurations: {:#04x}\n", self.bNumConfigurations).as_str());

        write!(f, "{}", res)
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct Device {
    pub container_id: String,
    pub driver_key_name: String,
    pub port_number: u8,
    pub descriptor: UsbDeviceDescriptor,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct Hub {
    pub hub_id: String,
    pub number_of_ports: u8,
    pub devices: Vec<Device>,
    pub parent_hub: Option<String>,
    pub descriptor: Option<UsbDeviceDescriptor>,
}
