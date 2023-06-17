#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ptr;
use std::env;
use std::path::Path;

fn main() {
    let path = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("path: {:?}", Path::new(&path));
    unsafe {
            let mut context_ptr: *mut libusb_context = ptr::null_mut();
            let result = libusb_init(&mut context_ptr);
            let mut usb_dev_list: *mut *mut libusb_device = ptr::null_mut();
            if result == 0 {
                let cnt = libusb_get_device_list(context_ptr, &mut usb_dev_list);
                println!("cnt: {}", cnt);
            }
            else {
                panic!("libusb initialization failed.");
            }
            libusb_free_device_list(usb_dev_list, 1); // Free the device list
            libusb_exit(context_ptr); // Clean up and exit the libusb context
        }
}
