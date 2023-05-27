use descriptor::Device;
use descriptor::Hub;
use descriptor::UsbDeviceDescriptor;
use driver::get_all_ids;
use helper::get_error;
use helper::get_mut_ptr;
use std::ffi::c_void;
use windows::core::Error;
use windows::core::HSTRING;
use windows::core::PCWSTR;
use windows::Win32::Devices::Usb::DeviceConnected;
use windows::Win32::Devices::Usb::NoDeviceConnected;
use windows::Win32::Devices::Usb::IOCTL_USB_GET_DESCRIPTOR_FROM_NODE_CONNECTION;
use windows::Win32::Devices::Usb::IOCTL_USB_GET_NODE_CONNECTION_DRIVERKEY_NAME;
use windows::Win32::Devices::Usb::IOCTL_USB_GET_NODE_CONNECTION_INFORMATION;
use windows::Win32::Devices::Usb::IOCTL_USB_GET_NODE_CONNECTION_NAME;
use windows::Win32::Devices::Usb::IOCTL_USB_GET_NODE_INFORMATION;
use windows::Win32::Devices::Usb::IOCTL_USB_GET_ROOT_HUB_NAME;
use windows::Win32::Devices::Usb::MAX_USB_STRING_LENGTH;
use windows::Win32::Devices::Usb::USB_DESCRIPTOR_REQUEST;
use windows::Win32::Devices::Usb::USB_NODE_CONNECTION_DRIVERKEY_NAME;
use windows::Win32::Devices::Usb::USB_NODE_CONNECTION_INFORMATION;
use windows::Win32::Devices::Usb::USB_NODE_CONNECTION_NAME;
use windows::Win32::Devices::Usb::USB_NODE_INFORMATION;
use windows::Win32::Devices::Usb::USB_ROOT_HUB_NAME;
use windows::Win32::Devices::Usb::USB_STRING_DESCRIPTOR_TYPE;
use windows::Win32::Foundation::BOOL;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::Storage::FileSystem::CreateFileW;
use windows::Win32::Storage::FileSystem::FILE_GENERIC_WRITE;
use windows::Win32::Storage::FileSystem::FILE_SHARE_WRITE;
use windows::Win32::Storage::FileSystem::OPEN_EXISTING;
use windows::Win32::Storage::FileSystem::SECURITY_ANONYMOUS;
use windows::Win32::System::IO::DeviceIoControl;

pub mod descriptor;
pub(crate) mod driver;
pub(crate) mod helper;

// https://learn.microsoft.com/en-us/samples/microsoft/windows-driver-samples/usbview-sample-application/

fn get_root_hub_name(handle: HANDLE) -> Result<String, String> {
    let retbytes = Some(0 as *mut u32);
    let mut outbuf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let outbuf_ptr = get_mut_ptr(&mut outbuf);

    let result = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_USB_GET_ROOT_HUB_NAME,
            None,
            0,
            Some(outbuf_ptr),
            (outbuf.len() * std::mem::size_of::<u16>()) as u32,
            retbytes,
            None,
        )
    };
    return if result == BOOL(1) {
        let start = (std::mem::size_of::<USB_ROOT_HUB_NAME>() - 2) / 2; // RootHubName so minus 2, divide by 2 for u16
        let b = String::from_utf16_lossy(&outbuf[start..]);
        let b = b.trim_end_matches('\0');
        Ok(b.to_string())
    } else {
        Err(get_error())
    };
}

fn get_number_of_ports(handle: HANDLE) -> Result<u8, String> {
    unsafe {
        let retbytes = Some(0 as *mut u32);
        let mut inbuf = USB_NODE_INFORMATION::default();
        inbuf.NodeType = windows::Win32::Devices::Usb::UsbHub;
        let inbuf_ptr = get_mut_ptr(&mut inbuf);

        let mut outbuf = USB_NODE_INFORMATION::default();
        let outbuf_ptr = get_mut_ptr(&mut outbuf);

        let result = DeviceIoControl(
            handle,
            IOCTL_USB_GET_NODE_INFORMATION,
            Some(inbuf_ptr),
            (std::mem::size_of::<USB_NODE_INFORMATION>()) as u32,
            Some(outbuf_ptr),
            (std::mem::size_of::<USB_NODE_INFORMATION>()) as u32,
            retbytes,
            None,
        );
        return if result == BOOL(1) {
            let number_of_hub_ports = outbuf.u.HubInformation.HubDescriptor.bNumberOfPorts;
            Ok(number_of_hub_ports)
        } else {
            Err(get_error())
        };
    };
}

fn open_device(hub: &mut String) -> Result<HANDLE, Error> {
    hub.insert_str(0, r"\\.\");
    let hub: PCWSTR = PCWSTR(HSTRING::from(hub.clone()).as_ptr());
    let x = unsafe {
        CreateFileW(
            hub,
            FILE_GENERIC_WRITE.0,
            FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            SECURITY_ANONYMOUS,
            None,
        )
    };
    return match x {
        Ok(handle) => Ok(handle),
        Err(err) => Err(err),
    };
}

fn get_descriptor(
    handle: HANDLE,
    port_number: u8,
    descriptor_id: u8,
    lang_id: u16,
) -> Option<(String, u16)> {
    if descriptor_id == 0 {
        return None;
    }
    // https://learn.microsoft.com/en-us/windows-hardware/drivers/usbcon/usb-string-descriptors

    let retbytes = Some(0 as *mut u32);
    let mut inbuf = USB_DESCRIPTOR_REQUEST::default();
    inbuf.ConnectionIndex = port_number as u32;
    inbuf.SetupPacket.wValue = ((USB_STRING_DESCRIPTOR_TYPE << 8) | descriptor_id as u32) as u16;
    inbuf.SetupPacket.wIndex = lang_id;
    inbuf.SetupPacket.wLength = MAX_USB_STRING_LENGTH as u16;
    let inbuf_ptr = get_mut_ptr(&mut inbuf);

    let mut outbuf: [u16; (MAX_USB_STRING_LENGTH) as usize] = [0; (MAX_USB_STRING_LENGTH) as usize];
    let outbuf_ptr = get_mut_ptr(&mut outbuf);

    let result = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_USB_GET_DESCRIPTOR_FROM_NODE_CONNECTION,
            Some(inbuf_ptr),
            (std::mem::size_of::<USB_DESCRIPTOR_REQUEST>()) as u32,
            Some(outbuf_ptr),
            (outbuf.len() * std::mem::size_of::<u16>()) as u32,
            retbytes,
            None,
        )
    };

    return if result == BOOL(1) {
        if lang_id == 0 {
            let first_lang = outbuf[0];
            Some(("".to_string(), first_lang))
        } else {
            let start = (std::mem::size_of::<USB_DESCRIPTOR_REQUEST>() as u32 - 1)
                / std::mem::size_of::<u16>() as u32; // Struct size is 13 bytes with data starting at byte 12. buffer is u16 so divide length by 2
            let b = String::from_utf16_lossy(&outbuf[(start + 1) as usize..]); // +1 for alignment
            let b = b.trim_end_matches('\0');
            Some((b.to_string(), 0))
        }
    } else {
        None
    };
}

fn get_string_fallback(
    handle: HANDLE,
    port_number: u8,
    string_id: u8,
    lang_id: u16,
) -> Option<String> {
    let tmp = get_descriptor(handle, port_number, string_id, lang_id);
    let result: Option<String>;
    if tmp == None {
        // If there is not localized descriptor try the first one from the list of supported languages
        let tmp = get_descriptor(handle, port_number, string_id, 0); // Get first language from descriptor
        result = match tmp {
            Some(code) => {
                let tmp: Option<(String, u16)> =
                    get_descriptor(handle, port_number, string_id, code.1); // Request again //TODO Check with virtualbox
                match tmp {
                    Some(code) => Some(code.0),
                    None => {
                        //println!("US FALLBACK MODE");
                        let tmp: Option<(String, u16)> =
                            get_descriptor(handle, port_number, string_id, 0x0409); // Fallback to us-en
                        match tmp {
                            Some(val) => Some(val.0),
                            None => None,
                        }
                    }
                }
            }
            None => None,
        };
    } else {
        result = match tmp {
            Some(val) => Some(val.0),
            None => None,
        }
    }
    return result;
}

fn get_secondary_hub_name(handle: HANDLE, index: u32) -> Result<String, String> {
    let retbytes = Some(0 as *mut u32);
    let mut inbuf = USB_NODE_CONNECTION_NAME::default();
    inbuf.ConnectionIndex = index;
    let input_ptr = get_mut_ptr(&mut inbuf);

    let mut outbuf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let outbuf_ptr = get_mut_ptr(&mut outbuf);

    let result = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_USB_GET_NODE_CONNECTION_NAME,
            Some(input_ptr),
            (std::mem::size_of::<USB_NODE_CONNECTION_NAME>()) as u32,
            Some(outbuf_ptr),
            (outbuf.len() * std::mem::size_of::<u16>()) as u32,
            retbytes,
            None,
        )
    };
    return if result == BOOL(1) {
        let start = (std::mem::size_of::<USB_NODE_CONNECTION_NAME>() as u32 - 2)
            / std::mem::size_of::<u16>() as u32; // Struct size is 10 bytes with data starting at byte 8. buffer is u16 so divide length by 2
        let b = String::from_utf16_lossy(&outbuf[start as usize..]);
        let b = b.trim_end_matches('\0');
        Ok(b.to_string())
    } else {
        Err(get_error())
    };
}

fn get_port_information(
    handle: HANDLE,
    port_number: u8,
    hubs: &mut Vec<Hub>,
    parent_hub: String,
    lang_id: u16,
) -> Result<UsbDeviceDescriptor, String> {
    let retbytes = Some(0 as *mut u32);
    let mut inbuf = USB_NODE_CONNECTION_INFORMATION::default();
    inbuf.ConnectionIndex = port_number as u32;
    let inbuf_ptr = get_mut_ptr(&mut inbuf);

    let mut outbuf = USB_NODE_CONNECTION_INFORMATION::default(); // This is working because USB_NODE_CONNECTION_INFORMATION has no stupid arrarys
    let outbuf_ptr = get_mut_ptr(&mut outbuf);

    let result = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_USB_GET_NODE_CONNECTION_INFORMATION,
            Some(inbuf_ptr),
            (std::mem::size_of::<USB_NODE_CONNECTION_INFORMATION>()) as u32,
            Some(outbuf_ptr),
            (std::mem::size_of::<USB_NODE_CONNECTION_INFORMATION>()) as u32,
            retbytes,
            None,
        )
    };
    if result == BOOL(1) {
        let connected = outbuf.ConnectionStatus;
        if connected == DeviceConnected {
            let desc = UsbDeviceDescriptor {
                bLength: outbuf.DeviceDescriptor.bLength,
                bDescriptorType: outbuf.DeviceDescriptor.bDescriptorType,
                bcdUSB: outbuf.DeviceDescriptor.bcdUSB,
                bDeviceClass: outbuf.DeviceDescriptor.bDeviceClass,
                bDeviceSubClass: outbuf.DeviceDescriptor.bDeviceSubClass,
                bDeviceProtocol: outbuf.DeviceDescriptor.bDeviceProtocol,
                bMaxPacketSize0: outbuf.DeviceDescriptor.bMaxPacketSize0,
                idVendor: outbuf.DeviceDescriptor.idVendor,
                idProduct: outbuf.DeviceDescriptor.idProduct,
                bcdDevice: outbuf.DeviceDescriptor.bcdDevice,
                iManufacturer: (
                    outbuf.DeviceDescriptor.iManufacturer,
                    get_string_fallback(
                        handle,
                        port_number,
                        outbuf.DeviceDescriptor.iManufacturer,
                        lang_id,
                    ),
                ),
                iProduct: (
                    outbuf.DeviceDescriptor.iProduct,
                    get_string_fallback(
                        handle,
                        port_number,
                        outbuf.DeviceDescriptor.iProduct,
                        lang_id,
                    ),
                ),
                iSerialNumber: (
                    outbuf.DeviceDescriptor.iSerialNumber,
                    get_string_fallback(
                        handle,
                        port_number,
                        outbuf.DeviceDescriptor.iSerialNumber,
                        lang_id,
                    ),
                ),
                bNumConfigurations: outbuf.DeviceDescriptor.bNumConfigurations,
            };

            if outbuf.DeviceIsHub.as_bool() {
                match get_secondary_hub_name(handle, outbuf.ConnectionIndex) {
                    Ok(hub_id) => {
                        match get_hub_devices(hub_id.clone(), hubs, lang_id) {
                            Ok(mut hub) => {
                                hub.parent_hub = Some(parent_hub);
                                hub.descriptor = Some(desc);
                                hubs.push(hub);
                                return Err(format!("Device is a hub which is added to the hub list instead of returning it"));
                            }
                            Err(err) => {
                                return Err(format!("Could not extract hub devices: {}", err));
                            }
                        };
                    }
                    Err(err) => {
                        return Err(format!("Could not extract hub name: {}", err));
                    }
                };
            } else {
                return Ok(desc);
            }
        } else if connected == NoDeviceConnected {
            return Err(format!("Port {} is not connected", port_number));
        } else {
            return Err(format!("Port {} is in transition state", port_number));
        }
    } else {
        return Err(get_error());
    }
}

fn get_driverkey_name(handle: HANDLE, port_number: u8) -> Result<String, String> {
    let retbytes: Option<*mut u32> = Some(0 as *mut u32);

    let mut inbuf = USB_NODE_CONNECTION_DRIVERKEY_NAME::default();
    inbuf.ConnectionIndex = port_number as u32;
    let inbuf_ptr: *mut c_void = get_mut_ptr(&mut inbuf);

    let mut outbuf: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let outbuf_ptr = get_mut_ptr(&mut outbuf);

    let result = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_USB_GET_NODE_CONNECTION_DRIVERKEY_NAME,
            Some(inbuf_ptr),
            (std::mem::size_of::<USB_NODE_CONNECTION_DRIVERKEY_NAME>()) as u32,
            Some(outbuf_ptr),
            (outbuf.len() * std::mem::size_of::<u16>()) as u32,
            retbytes,
            None,
        )
    };

    return if result == BOOL(1) {
        let start = (std::mem::size_of::<USB_NODE_CONNECTION_DRIVERKEY_NAME>() - 1) / 2; // DriverKeyName has 4 bytes, divide by 2 for u16
        let b = String::from_utf16_lossy(&outbuf[start..]);
        let b = b.trim_end_matches('\0');
        Ok(b.to_string())
    } else {
        Err(get_error())
    };
}

fn get_hub_devices(hub: String, hub_list: &mut Vec<Hub>, lang_id: u16) -> Result<Hub, String> {
    let mut hub_results = Vec::new();
    return match open_device(&mut hub.clone()) {
        Ok(hub_handle) => match get_number_of_ports(hub_handle) {
            Ok(number_of_ports) => {
                for port_number in 1..number_of_ports {
                    match get_port_information(
                        hub_handle,
                        port_number,
                        hub_list,
                        hub.clone(),
                        lang_id,
                    ) {
                        Ok(desc) => {
                            let mut device = Device {
                                port_number,
                                container_id: "".to_string(),
                                driver_key_name: "".to_string(),
                                descriptor: desc,
                            };
                            match get_driverkey_name(hub_handle, port_number) {
                                Ok(driverkey) => {
                                    device.driver_key_name = driverkey.clone();
                                    match get_all_ids() {
                                        Ok(result) => {
                                            for element in result {
                                                if element.0 == driverkey {
                                                    device.container_id = element.1;
                                                    hub_results.push(device);
                                                    break;
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            return Err(err);
                                        }
                                    };
                                }
                                Err(err) => {
                                    return Err(err);
                                }
                            };
                        }
                        Err(_err) => {
                            // NOTE: Ignore not connected/transition/ishub errors
                            //println!("Error: {}", err);
                        }
                    }
                }
                Ok(Hub {
                    hub_id: hub,
                    number_of_ports,
                    devices: hub_results,
                    parent_hub: None,
                    descriptor: None,
                })
            }
            Err(err) => Err(err),
        },
        Err(err) => Err(err.to_string()),
    };
}

pub fn get_all_hubs_with_devices(lang_id: u16) -> Vec<Hub> {
    let mut results: Vec<Hub> = Vec::new();
    for root_hub_number in 0..0xff {
        let root_hub = format!("\\\\.\\HCD{}", root_hub_number);
        match open_device(&mut root_hub.clone()) {
            Ok(handle) => match get_root_hub_name(handle) {
                Ok(hub) => match get_hub_devices(hub, &mut results, lang_id) {
                    Ok(hub) => {
                        results.push(hub);
                    }
                    Err(err) => {
                        println!("{}", err);
                    }
                },
                Err(err) => {
                    println!("Error: {}", err);
                }
            },
            Err(_err) => {
                continue; // Hub does not exist
            }
        }
    }
    results
}
