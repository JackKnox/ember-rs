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

pub struct DeviceConfig<'a> {
    pub debug_name: String,
    pub frame_allocator: Allocator,
    pub app_version: Version,
    pub required_modes: DeviceMode,
    pub optional_modes: DeviceMode,
    pub frames_in_flight: u32, 
    pub extensions: &'a[Box<dyn Extension>]
}

pub struct Queue(ffi::emgpu_queue);

pub struct Device {
    sys: ffi::emgpu_device,
}

trait Extension {}

impl Default for DeviceConfig<'_> {
    fn default() -> Self {
        DeviceConfig {
            debug_name: String::from("ember-rs GPU device"),
            frame_allocator: Allocator::system(),
            app_version: Version::new(0, 0, 1),
            required_modes: DeviceMode::Raster,
            optional_modes: DeviceMode::empty(),
            frames_in_flight: 3,
            extensions: &[],
        }
    }
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

    pub fn open_queue(&mut self) -> Result<Queue> {
        let mut queue: ffi::emgpu_queue = 0;
        let result = unsafe { 
            ffi::emgpu_device_open_queue(
                &mut self.sys as *mut ffi::emgpu_device, 
                &mut queue) };

        if result != ffi::em_result_EMBER_RESULT_OK {
            return Err(result.into());
        }

        Ok(Queue { 0: queue.into() })
    }

    pub fn submit(&mut self, command_buffer: CommandBuffer) -> Result<()> {
        todo!()
    }
}

pub struct CommandBuffer {
    sys: ffi::emgpu_command_buffer,
}

////////////////////////////////////////////////

pub mod ext {
    use crate::core::{Version, Allocator};
    use crate::gpu::{Device, Extension};
    use crate::window::Desktop;
    use crate::ffi;

    pub struct EmwinSurfaceExt {
        pub sys: ffi::emgpu_extension_desc,
        pub data: ffi::emgpu_emwin_surface_ext,
    }

    impl Extension for EmwinSurfaceExt {}

    #[repr(C)]
    struct EmwinSurfaceParams {
        desktop: *const ffi::emwin_desktop,
        out_extension: *mut ffi::emgpu_emwin_surface_ext,
    }

    pub struct EmwinSurfaceConfig {

    }

    impl EmwinSurfaceExt {
        pub fn register(desktop: &Desktop) -> Self {
            let mut ext = Self {
                sys: ffi::emgpu_extension_desc {
                    name: b"EMGPU_EXT_emwin_surface\0".as_ptr() as *const i8,
                    version: Version::ember_rs().into(),
                    optional: false,
                    user_data: ffi::emgpu_extension_user_data {
                        bytes: [0; 16],
                    },
                },
                data: unsafe { std::mem::zeroed() },
            };

            let params = EmwinSurfaceParams {
                desktop: desktop.sys as *const ffi::emwin_desktop,
                out_extension: &mut ext.data as *mut ffi::emgpu_emwin_surface_ext,
            };

            unsafe {
                std::ptr::copy_nonoverlapping(
                    &params as *const _ as *const u8,
                    ext.sys.user_data.bytes.as_mut_ptr(),
                    std::mem::size_of::<EmwinSurfaceParams>(),
                );
            }

            ext
        }

        pub fn create_surface(&self, device: &Device, allocator: &Allocator, config: &EmwinSurfaceConfig) {
            todo!()
        }
    }
}
