use std::mem;

use windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO;
use windows::{
    core::GUID,
    Win32::{
        Devices::{
            DeviceAndDriverInstallation::{
                SetupDiEnumDeviceInfo, SetupDiGetClassDevsW, SetupDiGetDevicePropertyW,
                DIGCF_ALLCLASSES, DIGCF_PRESENT, SP_DEVINFO_DATA,
            },
            Properties::{
                DEVPKEY_Device_ContainerId, DEVPKEY_Device_Driver, DEVPROPTYPE, DEVPROP_TYPE_GUID,
            },
        },
        Foundation::{GetLastError, BOOL, ERROR_NO_MORE_ITEMS, MAX_PATH},
    },
};

use crate::helper::get_error;

fn get_driver_id(
    device_info_set: HDEVINFO,
    dev_info_data: &SP_DEVINFO_DATA,
) -> Result<String, String> {
    let mut data_type: DEVPROPTYPE = DEVPROP_TYPE_GUID;
    let mut data: Vec<u8> = vec![0u8; MAX_PATH as usize];
    let buffer: Option<&mut [u8]> = Some(&mut data[..]);
    let ptr: *mut u32 = 0 as *mut u32;
    let reqsize: Option<*mut u32> = Some(ptr);

    let result = unsafe {
        SetupDiGetDevicePropertyW(
            device_info_set,
            dev_info_data,
            &DEVPKEY_Device_Driver,
            &mut data_type,
            buffer,
            reqsize,
            0,
        )
    };

    if result == BOOL(0) {
        //unsafe { SetupDiDestroyDeviceInfoList(device_info_set) };
        return Err(get_error());
    } else {
        let mut vec_u16: Vec<u16> = Vec::new();
        let mut i = 0;
        while i < data.len() {
            vec_u16.push((data[i + 1] as u16) << 8 | data[i] as u16);
            i += 2; // Convert to u16 manually because it is utf16
        }
        let b = String::from_utf16_lossy(&vec_u16);
        let b = b.trim_end_matches('\0');
        Ok(b.to_string())
    }
}

fn get_container_id(
    device_info_set: HDEVINFO,
    dev_info_data: &SP_DEVINFO_DATA,
) -> Result<String, String> {
    let mut data_type: DEVPROPTYPE = DEVPROP_TYPE_GUID;
    let mut data: Vec<u8> = vec![0u8; MAX_PATH as usize];
    let buffer: Option<&mut [u8]> = Some(&mut data[..]);
    let ptr: *mut u32 = 0 as *mut u32;
    let reqsize: Option<*mut u32> = Some(ptr);

    let result = unsafe {
        SetupDiGetDevicePropertyW(
            device_info_set,
            dev_info_data,
            &DEVPKEY_Device_ContainerId,
            &mut data_type,
            buffer,
            reqsize,
            0,
        )
    };

    if result == BOOL(0) {
        //unsafe { SetupDiDestroyDeviceInfoList(device_info_set) };
        return Err(get_error());
    } else {
        let data1 = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let data2 = u16::from_le_bytes(data[4..6].try_into().unwrap());
        let data3 = u16::from_le_bytes(data[6..8].try_into().unwrap());
        let mut data4: [u8; 8] = [0; 8];
        data4.copy_from_slice(&data[8..16]);

        let x = GUID::from_values(data1, data2, data3, data4);
        let x = format!("{{{:?}}}", x).to_lowercase();
        //println!("GUID {}", x);
        return Ok(x);
    }
}

pub(crate) fn get_all_ids() -> Result<Vec<(String, String)>, String> {
    let devices = unsafe {
        SetupDiGetClassDevsW(
            None,
            windows::w!("USB"),
            None,
            DIGCF_PRESENT | DIGCF_ALLCLASSES,
        )
    };
    let mut results = Vec::new();
    match devices {
        Ok(device_info_set) => {
            if device_info_set.is_invalid() {
                return Err("Could not get devices".to_string());
            }

            // Enumerate devices in the device information set
            let mut index: u32 = 0;
            loop {
                let mut dev_info_data: SP_DEVINFO_DATA = unsafe { mem::zeroed() };
                dev_info_data.cbSize = mem::size_of::<SP_DEVINFO_DATA>() as u32;
                let result =
                    unsafe { SetupDiEnumDeviceInfo(device_info_set, index, &mut dev_info_data) };
                if result == BOOL(0) {
                    let x = unsafe { GetLastError() };
                    if x == ERROR_NO_MORE_ITEMS {
                        break;
                    }
                    //println!("Error {:?}", x);
                    continue;
                }
                match get_driver_id(device_info_set, &dev_info_data) {
                    Ok(guid) => {
                        match get_container_id(device_info_set, &dev_info_data) {
                            Ok(cguid) => {
                                results.push((guid, cguid));
                            }
                            Err(_err) => {
                                //println!("{}", err)
                            }
                        };
                    }
                    Err(_err) => {
                        //println!("{}", err)
                    }
                };
                index += 1;
            }
        }
        Err(_err) => {
            //println!("{:?}", err);
        }
    }
    return Ok(results);
}
