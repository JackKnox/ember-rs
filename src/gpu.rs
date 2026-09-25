use crate::core::{Allocator, Result, Version};
use crate::ffi;

use bitflags::bitflags;

use std::ffi::CString;

////////////////////////////////////////////////

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct DeviceMode: u32 {
        const Raster            = 0b00000001;
        const Compute           = 0b00000010;
        const Raytrace          = 0b00000100;
        const Transfer          = 0b00001000;
        const Present           = 0b00010000;
        const Validation        = 0b00100000;
        const PowerSaving       = 0b01000000;
        const SamplerAnisotropy = 0b10000000;
    }
}

pub enum DeviceType {
    Other,
    Intergrated,
    Discrete,
    Virtual,
    CPU,
}

pub struct DeviceCapabilities<'a> {
    pub device_name: &'a str,
    pub device_type: DeviceType,
    pub enabled_modes: DeviceMode,
    pub driver_version: Version,
    pub max_anisotropy: f32,
    // pub vendor_signiture: VendorSigniture,
}

pub struct DeviceConfig {
    debug_name: String,
    frame_allocator: Allocator,
    app_version: Version,
    required_modes: DeviceMode,
    optional_modes: DeviceMode,
    frames_in_flight: u32, //extensions: Vec<Box<dyn DeviceExtension>>
}

pub struct Queue(ffi::emgpu_queue);

pub struct Device {
    sys: ffi::emgpu_device,
}

pub struct CommandBuffer {
    sys: ffi::emgpu_command_buffer,
}

impl Device {
    pub fn init(allocator: &Allocator, config: &DeviceConfig) -> Result<Device> {
        let debug_name = CString::new(config.debug_name.clone())
            .expect("Device debug name contained a NULL byte");

        let c_config = ffi::emgpu_device_config {
            api_next: std::ptr::null_mut(),
            debug_name: debug_name.as_ptr(),
            frame_allocator: unsafe { *config.frame_allocator.sys.get() },
            app_version: config.app_version.into(),
            required_modes: config.required_modes.bits(),
            optional_modes: config.optional_modes.bits(),
            frames_in_flight: config.frames_in_flight,
            extension_count: 0,
            extensions: std::ptr::null_mut(),
        };
        let mut device = Device {
            sys: unsafe { std::mem::zeroed() },
        };
        let result = unsafe {
            ffi::emgpu_device_init(
                allocator.sys.get() as *mut ffi::em_allocator,
                &c_config as *const ffi::emgpu_device_config,
                &mut device.sys as *mut ffi::emgpu_device,
            )
        };

        if result != ffi::em_result_EMBER_RESULT_OK {
            return Err(result.into());
        }
        Ok(device)
    }

    pub fn shutdown(&mut self, allocator: &Allocator) {
        unsafe { 
            ffi::emgpu_device_shutdown(
                allocator.sys.get() as *mut ffi::em_allocator, 
                &mut self.sys as *mut ffi::emgpu_device);
        }
    }

    pub fn capabilities(&self) -> DeviceCapabilities<'_> {
        todo!()
    }

    pub fn submit(&mut self, command_buffer: CommandBuffer) -> Result<()> {
        todo!()
    }
}
