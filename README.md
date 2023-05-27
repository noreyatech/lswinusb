# lswinusb - A Rust library for listing USB Descriptors on Windows

**This library is in an experimental stage and needs testing, it is not ready for crates.io publishing**

## The problem
Unlike Linux and Mac, Windows does not decode, store or provide the full USB descriptor of the connected devices.

The only way to access the descriptor is to send USB requests directly to the devices via Win32::System::IO::DeviceIoControl.

This library uses the Win32 API to read and decode the USB descriptors. It provides them as serializable structures.

## The goal
The long term goal of this library is to provide an easy way to read the complete USB related information provided by the DeviceIoControl API.  
Currently only the device descriptor is supported, other information like interface and configuration descriptors 
as well as USB hub information are not implemented (but can be easily integrated).

This library has no intention to implement "write/control" commands.

## Contribution
**Test reports, patches and problems are welcome!**  
Since this lib is not under active development, feature requests will not be processed. Please implement it yourself or contact me if you are willing to pay for it.

## (Possible) issues
- Cross compilation is not possible, must be compiled on Windows
- Accessing the Win32 means *unsafe* code  
  The lib may leak memory
- Tested is Windows 7, 8.1 and 10 64-bit  
  32-bit as well as arm32/64 support is untested 


## Usage example
Cargo.toml:  
```toml
[dependencies]
lswinusb = { package = "lswinusb", git = "https://github.com/noreyatech/lswinusb.git", branch = "master", features = ["serde"] }
serde_json = { version = "1.0"}

[dependencies.windows]
version = "0.48"
features = ["Win32_Globalization"]

```

main.rs:
```rust
use lswinusb::get_all_hubs_with_devices;
use windows::Win32::Globalization::GetSystemDefaultLangID;

fn main() {
    let lang_id;
    unsafe { lang_id = GetSystemDefaultLangID() } // Windows uses localized descriptors...
    let res = serde_json::to_string_pretty(&get_all_hubs_with_devices(lang_id))
        .expect("This must be a struct");
    println!("{}", res);
}
```

Sample output:
```json
[
  {
    "hub_id": "USB#ROOT_HUB30#4&24054718&0&0#{f18a0e88-c30c-11d0-8815-00a0c906bed8}",
    "number_of_ports": 14,
    "devices": [
      {
        "container_id": "{dd2b33e2-f686-11ed-b1c4-806e6f6e6963}",
        "driver_key_name": "{745a17a0-74d3-11d0-b6fe-00a0c90f57da}\\0001",
        "port_number": 1,
        "descriptor": {
          "bLength": 18,
          "bDescriptorType": 1,
          "bcdUSB": 272,
          "bDeviceClass": 0,
          "bDeviceSubClass": 0,
          "bDeviceProtocol": 0,
          "bMaxPacketSize0": 8,
          "idVendor": 33006,
          "idProduct": 33,
          "bcdDevice": 256,
          "iManufacturer": [
            1,
            "VirtualBox"
          ],
          "iProduct": [
            3,
            "USB Tablet"
          ],
          "iSerialNumber": [
            0,
            null
          ],
          "bNumConfigurations": 1
        }
      }
    ],
    "parent_hub": null,
    "descriptor": null
  }
]
```

## Other
If you want a comparison tool which uses the same API you can test the GUI utility [usbtreeview](https://www.uwe-sieber.de/usbtreeview_e.html).