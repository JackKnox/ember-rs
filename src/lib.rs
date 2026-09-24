mod ffi {
    #![allow(non_snake_case)]
    #![allow(non_camel_case_types)]
    #![allow(non_upper_case_globals)]
    #![allow(dead_code)]

    include!("ffi.rs");
}

pub mod core;
pub mod gpu;
pub mod window;
