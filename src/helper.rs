use std::ffi::c_void;
use windows::Win32::Foundation::GetLastError;

pub(crate) fn get_error() -> String {
    format!("{:?}", unsafe { GetLastError() })
}

pub(crate) fn get_mut_ptr<T>(buf: &mut T) -> *mut c_void {
    let ptr: *mut c_void = buf as *mut _ as *mut c_void;
    return ptr;
}
