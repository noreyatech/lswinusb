use std::mem;
use windows::core::HRESULT;

use windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO;
use windows::{
    Win32::{
        Devices::{
            DeviceAndDriverInstallation::{
                DIGCF_ALLCLASSES, DIGCF_PRESENT, SP_DEVINFO_DATA, SetupDiEnumDeviceInfo,
                SetupDiGetClassDevsW, SetupDiGetDevicePropertyW,
            },
            Properties::{
                DEVPKEY_Device_ContainerId, DEVPKEY_Device_Driver, DEVPROP_TYPE_GUID, DEVPROPTYPE,
            },
        },
        Foundation::{ERROR_NO_MORE_ITEMS, MAX_PATH},
    },
    core::GUID,
};

fn get_driver_id(
    device_info_set: HDEVINFO,
    dev_info_data: &SP_DEVINFO_DATA,
) -> Result<String, String> {
    let mut data_type: DEVPROPTYPE = DEVPROP_TYPE_GUID;
    let mut data: Vec<u8> = vec![0u8; MAX_PATH as usize];
    let buffer: Option<&mut [u8]> = Some(&mut data[..]);
    let ptr: *mut u32 = std::ptr::null_mut::<u32>();
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

    match result {
        Ok(_) => {
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
        Err(err) => {
            Err(err.to_string())
            //Err(get_error())
        }
    }
}

fn get_container_id(
    device_info_set: HDEVINFO,
    dev_info_data: &SP_DEVINFO_DATA,
) -> Result<String, String> {
    let mut data_type: DEVPROPTYPE = DEVPROP_TYPE_GUID;
    let mut data: Vec<u8> = vec![0u8; MAX_PATH as usize];
    let buffer: Option<&mut [u8]> = Some(&mut data[..]);
    let ptr: *mut u32 = std::ptr::null_mut::<u32>();
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
    match result {
        Ok(_) => {
            let data1 = u32::from_le_bytes(data[0..4].try_into().unwrap());
            let data2 = u16::from_le_bytes(data[4..6].try_into().unwrap());
            let data3 = u16::from_le_bytes(data[6..8].try_into().unwrap());
            let mut data4: [u8; 8] = [0; 8];
            data4.copy_from_slice(&data[8..16]);

            let x = GUID::from_values(data1, data2, data3, data4);
            let x = format!("{{{:?}}}", x).to_lowercase();
            //println!("GUID {}", x);
            Ok(x)
        }
        Err(err) => {
            Err(err.to_string())
            //Err(get_error())
        }
    }
}

pub(crate) fn get_all_ids() -> Result<Vec<(String, String)>, String> {
    let devices = unsafe {
        SetupDiGetClassDevsW(
            None,
            windows::core::w!("USB"),
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

                match result {
                    Ok(_) => {
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
                    Err(err) => {
                        // let x = unsafe { GetLastError() };
                        //println!("Error {:?}", x);
                        if err.code() == HRESULT::from(ERROR_NO_MORE_ITEMS) {
                            break;
                        }
                        continue;
                    }
                }
            }
        }
        Err(_err) => {
            //println!("{:?}", err);
        }
    }
    Ok(results)
}
