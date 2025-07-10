use lswinusb::{get_all_hubs_with_devices, get_system_default_language};

fn main() {
    let lang_id = get_system_default_language();
    let res = serde_json::to_string_pretty(&get_all_hubs_with_devices(lang_id))
        .expect("This must be a struct");
    println!("{}", res);
}
