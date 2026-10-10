use crate::core::{Allocator, Result, Version};
use crate::core::{Format, FormatFlags, DataType};
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
    pub extensions: &'a[&'a dyn Extension]
}

pub struct Queue(ffi::emgpu_queue);

pub struct Device {
    sys: ffi::emgpu_device,
}

pub trait Extension {}

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
                &mut device.sys as *mut ffi::emgpu_device)
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

    pub fn submit(&mut self, queue: Queue, command_buffer: CommandBuffer) -> Result<()> {
        todo!()
    }
}

pub struct CommandBuffer {
    data: Vec<u8>,
    current_resource_idx: u32,
}

pub struct LocalResource(u32);

pub struct LocalFramebuffer(u32);

pub enum LoadOp {
    Load, Store, DontCore
}

pub enum StoreOp {
    Store, DontCare
}

pub struct Colour(pub u32);

pub struct RenderpassColourAttachment {
    pub framebuffer: LocalFramebuffer,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
    pub colour: Colour,
    pub presentable: bool
}

pub struct RenderpassConfig<'a> {
    pub render_origin: [u32; 2],
    pub render_size: [u32; 2],
    pub colour_attachments: &'a[RenderpassColourAttachment],
}

#[repr(u32)]
#[derive(Clone, Copy)]
enum CommandType {
    Empty = 0,
    BeginComputePass,
    Dispatch,
    EndComputePass,
    BeginRenderPass,
    EndRenderPass,
    SetViewport,
    SetScissor,
    BindRasterPipeline,
    BindIndexBuffer,
    Draw,
    EmptyResource,
    ImportTexture,
    AcquireSurface,
    ExportResources,
    ImportResources,
    ColourAttachments,
    BindVertexBuffers,
}

#[repr(C)]
struct CommandHeader {
    pub cmd_type: CommandType,
    pub cmd_size: u64
}

impl CommandHeader {
    fn write_to(&self, dst: &mut [u8]) {
        const SIZE: usize = 16;

        assert!(dst.len() >= SIZE);

        dst[0..4].copy_from_slice(&(self.cmd_type as u32).to_le_bytes());
        dst[4..8].fill(0); // padding
        dst[8..16].copy_from_slice(&self.cmd_size.to_le_bytes());
    }
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(4),
            current_resource_idx: 0,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    fn alloc(&mut self, ty: CommandType, payload_size: usize) -> &mut [u8] {
        let offset = align_up(self.data.len(), 16);
        let total_size = size_of::<CommandHeader>() + payload_size;

        let end = offset + total_size;

        if self.data.len() < end {
            self.data.resize(end, 0);
        }

        let header = CommandHeader {
            cmd_type: ty,
            cmd_size: total_size as u64,
        };

        header.write_to(&mut self.data[offset..offset + size_of::<CommandHeader>()]);

        &mut self.data[
            offset + size_of::<CommandHeader>()
                ..offset + total_size
        ]
    }
}

fn align_up(value: usize, alignment: usize) -> usize {
    debug_assert!(alignment.is_power_of_two());

    (value + alignment - 1) & !(alignment - 1)
}

impl CommandBuffer {
    pub fn create(device: &Device) -> Result<CommandBuffer> {
        todo!()
    }

    pub fn empty_resource(&mut self) -> LocalResource {
        todo!()
    }

    pub fn acquire_surface(&mut self, surface: &Surface) -> LocalFramebuffer {
        todo!()
    }

    pub fn begin_renderpass(&mut self, config: &RenderpassConfig) {
        todo!()
    }

    pub fn set_viewport(&mut self, origin: [u64; 2], size: [u64; 2], min_depth: f32, max_depth: f32) {
        todo!()
    }

    pub fn set_scissor(&mut self, origin: [u64; 2], size: [u64; 2]) {
        todo!()
    }

    pub fn end_renderpass(&mut self) {
        todo!()
    }
}

pub struct Surface {
    pub sys: ffi::emgpu_surface,
}

impl Surface {
    pub fn resize(&mut self, device: &mut Device, new_size: [u64; 2]) -> Result<Surface> {
        todo!()
    }

    pub fn destroy(&mut self, device: &mut Device, allocator: &Allocator) {
        todo!()
    }
}

pub enum FilterType {
    Nearest,
    Linear
}

pub enum AddressMode {
    Repeat, 
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
    MirrorClampToEdge,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct TextureUsage: u32 {
        const Storage       = 0b00000001;
        const Sampled       = 0b00000010;
        const TransferSrc   = 0b00000100;
        const TransferDst   = 0b00001000;
        const AttachmentDst = 0b00010000;

    }
}

pub struct TextureConfig {
    pub image_format: GpuFormat,
    pub filter_type: FilterType,
    pub address_mode: AddressMode,
    pub usage: TextureUsage,
    pub size: [u64; 2],
    pub max_anisotropy: f32,
}

pub struct Texture {
    sys: ffi::emgpu_texture,
}

impl Default for TextureConfig {
    fn default() -> Self {
        TextureConfig {
            image_format: GpuFormat::RGBA8_SRGB,
            filter_type: FilterType::Nearest,
            address_mode: AddressMode::Repeat,
            usage: TextureUsage::Sampled,
            size: [ 0, 0 ],
            max_anisotropy: 0.0
        }
    }
}

////////////////////////////////////////////////

pub mod ext {
    use crate::core::{Version, Allocator, Result};
    use crate::gpu::{Device, Extension, GpuFormat, Surface, TextureUsage};
    use crate::window::{Window, Desktop};
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

    pub struct EmwinSurfaceConfig<'a> {
        pub preferred_format: GpuFormat,
        pub force_format: bool,
        pub min_texture_count: u32,
        pub usage: TextureUsage,
        pub window: Option<&'a Window>,
    }

    impl Default for EmwinSurfaceConfig<'_> {
        fn default() -> Self {
            EmwinSurfaceConfig {
                preferred_format: GpuFormat::BGR8_UNORM,
                force_format: false,
                min_texture_count: 3,
                usage: TextureUsage::AttachmentDst,
                window: None,
            }
        }
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

        pub fn create_surface(&self, device: &mut Device, allocator: &Allocator, config: &EmwinSurfaceConfig) -> Result<Surface> {
            let mut c_config = ffi::emgpu_emwin_surface_config {
                preferred_format: config.preferred_format.0.0,
                force_format: config.force_format,
                min_texture_count: config.min_texture_count,
                usage: config.usage.bits(),
                window: &config.window.unwrap().sys as *const ffi::emwin_window,
            };
            let mut surface = Surface {
                sys: unsafe { std::mem::zeroed() },
            };

            let result = unsafe {
                (self.data.create_surface.unwrap())(
                    &mut device.sys as *mut ffi::emgpu_device,
                    allocator.sys.get() as *mut ffi::em_allocator,
                    &mut c_config as *mut ffi::emgpu_emwin_surface_config,
                    &mut surface.sys as *mut ffi::emgpu_surface) 
            };
            
            if result != ffi::em_result_EMBER_RESULT_OK {
                return Err(result.into());
            }
            Ok(surface)
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GpuFormat(pub Format);

impl GpuFormat {
    pub const UNDEFINED: Self = Self(Format(0));

    /* 8-bit integer */

    pub const R8_UINT: Self =
        Self(Format::new(DataType::Uint, 8, 1, FormatFlags::empty()));
    pub const RG8_UINT: Self =
        Self(Format::new(DataType::Uint, 8, 2, FormatFlags::empty()));
    pub const RGB8_UINT: Self =
        Self(Format::new(DataType::Uint, 8, 3, FormatFlags::empty()));
    pub const RGBA8_UINT: Self =
        Self(Format::new(DataType::Uint, 8, 4, FormatFlags::empty()));

    pub const R8_SINT: Self =
        Self(Format::new(DataType::Sint, 8, 1, FormatFlags::empty()));
    pub const RG8_SINT: Self =
        Self(Format::new(DataType::Sint, 8, 2, FormatFlags::empty()));
    pub const RGB8_SINT: Self =
        Self(Format::new(DataType::Sint, 8, 3, FormatFlags::empty()));
    pub const RGBA8_SINT: Self =
        Self(Format::new(DataType::Sint, 8, 4, FormatFlags::empty()));

    /* 8-bit normalized */

    pub const R8_UNORM: Self =
        Self(Format::new(DataType::Uint, 8, 1, FormatFlags::NORMALIZED));
    pub const RG8_UNORM: Self =
        Self(Format::new(DataType::Uint, 8, 2, FormatFlags::NORMALIZED));
    pub const RGB8_UNORM: Self =
        Self(Format::new(DataType::Uint, 8, 3, FormatFlags::NORMALIZED));
    pub const RGBA8_UNORM: Self =
        Self(Format::new(DataType::Uint, 8, 4, FormatFlags::NORMALIZED));

    pub const R8_SNORM: Self =
        Self(Format::new(DataType::Sint, 8, 1, FormatFlags::NORMALIZED));
    pub const RG8_SNORM: Self =
        Self(Format::new(DataType::Sint, 8, 2, FormatFlags::NORMALIZED));
    pub const RGB8_SNORM: Self =
        Self(Format::new(DataType::Sint, 8, 3, FormatFlags::NORMALIZED));
    pub const RGBA8_SNORM: Self =
        Self(Format::new(DataType::Sint, 8, 4, FormatFlags::NORMALIZED));

    /* 16-bit */

    pub const R16_UINT: Self =
        Self(Format::new(DataType::Uint, 16, 1, FormatFlags::empty()));
    pub const RG16_UINT: Self =
        Self(Format::new(DataType::Uint, 16, 2, FormatFlags::empty()));
    pub const RGB16_UINT: Self =
        Self(Format::new(DataType::Uint, 16, 3, FormatFlags::empty()));
    pub const RGBA16_UINT: Self =
        Self(Format::new(DataType::Uint, 16, 4, FormatFlags::empty()));

    pub const R16_SINT: Self =
        Self(Format::new(DataType::Sint, 16, 1, FormatFlags::empty()));
    pub const RG16_SINT: Self =
        Self(Format::new(DataType::Sint, 16, 2, FormatFlags::empty()));
    pub const RGB16_SINT: Self =
        Self(Format::new(DataType::Sint, 16, 3, FormatFlags::empty()));
    pub const RGBA16_SINT: Self =
        Self(Format::new(DataType::Sint, 16, 4, FormatFlags::empty()));

    pub const R16_FLOAT: Self =
        Self(Format::new(DataType::Float, 16, 1, FormatFlags::empty()));
    pub const RG16_FLOAT: Self =
        Self(Format::new(DataType::Float, 16, 2, FormatFlags::empty()));
    pub const RGB16_FLOAT: Self =
        Self(Format::new(DataType::Float, 16, 3, FormatFlags::empty()));
    pub const RGBA16_FLOAT: Self =
        Self(Format::new(DataType::Float, 16, 4, FormatFlags::empty()));

    /* 32-bit */

    pub const R32_UINT: Self =
        Self(Format::new(DataType::Uint, 32, 1, FormatFlags::empty()));
    pub const RG32_UINT: Self =
        Self(Format::new(DataType::Uint, 32, 2, FormatFlags::empty()));
    pub const RGB32_UINT: Self =
        Self(Format::new(DataType::Uint, 32, 3, FormatFlags::empty()));
    pub const RGBA32_UINT: Self =
        Self(Format::new(DataType::Uint, 32, 4, FormatFlags::empty()));

    pub const R32_SINT: Self =
        Self(Format::new(DataType::Sint, 32, 1, FormatFlags::empty()));
    pub const RG32_SINT: Self =
        Self(Format::new(DataType::Sint, 32, 2, FormatFlags::empty()));
    pub const RGB32_SINT: Self =
        Self(Format::new(DataType::Sint, 32, 3, FormatFlags::empty()));
    pub const RGBA32_SINT: Self =
        Self(Format::new(DataType::Sint, 32, 4, FormatFlags::empty()));

    pub const R32_FLOAT: Self =
        Self(Format::new(DataType::Float, 32, 1, FormatFlags::empty()));
    pub const RG32_FLOAT: Self =
        Self(Format::new(DataType::Float, 32, 2, FormatFlags::empty()));
    pub const RGB32_FLOAT: Self =
        Self(Format::new(DataType::Float, 32, 3, FormatFlags::empty()));
    pub const RGBA32_FLOAT: Self =
        Self(Format::new(DataType::Float, 32, 4, FormatFlags::empty()));

    /* sRGB */

    pub const R8_SRGB: Self =
        Self(Format::new(DataType::Uint, 8, 1, FormatFlags::SRGB));
    pub const RG8_SRGB: Self =
        Self(Format::new(DataType::Uint, 8, 2, FormatFlags::SRGB));
    pub const RGB8_SRGB: Self =
        Self(Format::new(DataType::Uint, 8, 3, FormatFlags::SRGB));
    pub const RGBA8_SRGB: Self =
        Self(Format::new(DataType::Uint, 8, 4, FormatFlags::SRGB));

    /* BGR / BGRA */

    pub const BGR8_UNORM: Self =
        Self(Format::new(
            DataType::Uint,
            8,
            3,
            FormatFlags::from_bits_retain(
                FormatFlags::NORMALIZED.bits() | FormatFlags::BGRA.bits(),
            ),
        ));

    pub const BGR8_SNORM: Self =
        Self(Format::new(
            DataType::Sint,
            8,
            3,
            FormatFlags::from_bits_retain(
                FormatFlags::NORMALIZED.bits() | FormatFlags::BGRA.bits(),
            ),
        ));

    pub const BGR8_UINT: Self =
        Self(Format::new(DataType::Uint, 8, 3, FormatFlags::BGRA));

    pub const BGR8_SINT: Self =
        Self(Format::new(DataType::Sint, 8, 3, FormatFlags::BGRA));

    pub const BGR8_SRGB: Self =
        Self(Format::new(
            DataType::Uint,
            8,
            3,
            FormatFlags::from_bits_retain(
                FormatFlags::SRGB.bits() | FormatFlags::BGRA.bits(),
            ),
        ));

    pub const BGRA8_UNORM: Self =
        Self(Format::new(
            DataType::Uint,
            8,
            4,
            FormatFlags::from_bits_retain(
                FormatFlags::NORMALIZED.bits() | FormatFlags::BGRA.bits(),
            ),
        ));

    pub const BGRA8_SNORM: Self =
        Self(Format::new(
            DataType::Sint,
            8,
            4,
            FormatFlags::from_bits_retain(
                FormatFlags::NORMALIZED.bits() | FormatFlags::BGRA.bits(),
            ),
        ));

    pub const BGRA8_UINT: Self =
        Self(Format::new(DataType::Uint, 8, 4, FormatFlags::BGRA));

    pub const BGRA8_SINT: Self =
        Self(Format::new(DataType::Sint, 8, 4, FormatFlags::BGRA));

    pub const BGRA8_SRGB: Self =
        Self(Format::new(
            DataType::Uint,
            8,
            4,
            FormatFlags::from_bits_retain(
                FormatFlags::SRGB.bits() | FormatFlags::BGRA.bits(),
            ),
        ));

    /* Depth / stencil */

    pub const D16_UNORM: Self =
        Self(Format::new(DataType::Uint, 16, 1, FormatFlags::DEPTH));

    pub const D24_UNORM: Self =
        Self(Format::new(DataType::Uint, 24, 1, FormatFlags::DEPTH));

    pub const D32_FLOAT: Self =
        Self(Format::new(DataType::Float, 32, 1, FormatFlags::DEPTH));

    pub const D24_UNORM_S8_UINT: Self =
        Self(Format::new(
            DataType::Uint,
            32,
            2,
            FormatFlags::from_bits_retain(
                FormatFlags::DEPTH.bits() | FormatFlags::STENCIL.bits(),
            ),
        ));

    pub const D32_FLOAT_S8_UINT: Self =
        Self(Format::new(
            DataType::Float,
            40,
            2,
            FormatFlags::from_bits_retain(
                FormatFlags::DEPTH.bits() | FormatFlags::STENCIL.bits(),
            ),
        ));

    /* Vectors */

    pub const VEC2_FLOAT32: Self =
        Self(Format::new(DataType::Float, 32, 2, FormatFlags::empty()));
    pub const VEC3_FLOAT32: Self =
        Self(Format::new(DataType::Float, 32, 3, FormatFlags::empty()));
    pub const VEC4_FLOAT32: Self =
        Self(Format::new(DataType::Float, 32, 4, FormatFlags::empty()));

    pub const VEC2_UINT32: Self =
        Self(Format::new(DataType::Uint, 32, 2, FormatFlags::empty()));
    pub const VEC3_UINT32: Self =
        Self(Format::new(DataType::Uint, 32, 3, FormatFlags::empty()));
    pub const VEC4_UINT32: Self =
        Self(Format::new(DataType::Uint, 32, 4, FormatFlags::empty()));

    pub const VEC2_SINT32: Self =
        Self(Format::new(DataType::Sint, 32, 2, FormatFlags::empty()));
    pub const VEC3_SINT32: Self =
        Self(Format::new(DataType::Sint, 32, 3, FormatFlags::empty()));
    pub const VEC4_SINT32: Self =
        Self(Format::new(DataType::Sint, 32, 4, FormatFlags::empty()));
}

impl GpuFormat {
    pub const fn format(self) -> Format {
        self.0
    }

    pub const fn flags(self) -> FormatFlags {
        self.0.flags()
    }

    pub const fn data_type(self) -> DataType {
        self.0.data_type()
    }

    pub const fn bytes(self) -> u32 {
        self.0.bytes()
    }

    pub const fn channels(self) -> u32 {
        self.0.channels()
    }

    pub const fn size(self) -> u32 {
        self.0.size()
    }
}
