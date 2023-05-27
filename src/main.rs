use lswinusb::get_all_hubs_with_devices;
use windows::Win32::Globalization::GetSystemDefaultLangID;

fn main() {
    let lang_id;
    unsafe { lang_id = GetSystemDefaultLangID() } // Windows uses localized descriptors...
    let res = serde_json::to_string_pretty(&get_all_hubs_with_devices(lang_id))
        .expect("This must be a struct");
    println!("{}", res);
}
