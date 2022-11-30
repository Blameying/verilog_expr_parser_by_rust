use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::{c_char, c_uint};

#[link(name = "espresso", kind = "static")]
extern "C" {
    pub fn run_espresso_from_data(
        data: *const *const c_char,
        length: c_uint,
        ret_length: *mut c_uint,
    ) -> *mut *const c_char;
    pub fn run_espresso_from_path(
        path: *const c_char,
        ret_length: *mut c_uint,
    ) -> *mut *const c_char;
}

pub fn espresso_minimizer(data: Vec<String>) -> Vec<String> {
    let mut cstrs: Vec<CString> = Vec::new();
    let mut input: Vec<*const c_char> = Vec::new();

    for d in data.iter() {
        let mut new_data = d.clone();
        new_data.retain(|c| c != '\0');
        cstrs.push(CString::new(new_data.as_bytes()).unwrap());
    }

    for d in cstrs.iter() {
        input.push(d.as_ptr());
    }

    let mut result_length: c_uint = 0;
    let result: *mut *const c_char = unsafe {
        run_espresso_from_data(input.as_ptr(), input.len() as c_uint, &mut result_length)
    };

    let ret: Vec<*const c_char> =
        unsafe { Vec::from_raw_parts(result, result_length as usize, result_length as usize) };

    let mut ret_string: Vec<String> = Vec::new();
    for d in ret.iter() {
        unsafe {
            let c: &CStr = CStr::from_ptr(*d);
            ret_string.push(String::from(c.to_str().unwrap()));
        }
    }

    println!("{:?}", result_length);
    ret_string
}
