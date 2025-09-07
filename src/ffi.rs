use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn semverx_parse(version_str: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(version_str) };
    let version = c_str.to_str().unwrap_or("");
    
    // Parse and return
    let result = format!("Parsed: {}", version);
    CString::new(result).unwrap().into_raw()
}
