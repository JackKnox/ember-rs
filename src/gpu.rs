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

#[derive(Clone, Copy)]
pub struct Queue(ffi::emgpu_queue);

pub struct Device {
    sys: ffi::emgpu_device,
}

pub trait Extension {
    fn desc(&self) -> &ffi::emgpu_extension_desc;
    fn prepare(&self);
}

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
    pub fn init(
        allocator: &Allocator,
        config: &DeviceConfig,
    ) -> Result<Device> {
        let debug_name = CString::new(config.debug_name.clone())
            .expect("Device debug name contained a NULL byte");

        // Refresh all extension output pointers before copying descriptors.
        for ext in config.extensions {
            ext.prepare();
        }

        let mut raw_descs: Vec<ffi::emgpu_extension_desc> = config.extensions
            .iter()
            .map(|ext| *ext.desc())
            .collect();

        let c_config = ffi::emgpu_device_config {
            api_next: std::ptr::null_mut(),
            debug_name: debug_name.as_ptr(),
            frame_allocator: unsafe { *config.frame_allocator.sys.get() },
            app_version: config.app_version.into(),
            required_modes: config.required_modes.bits(),
            optional_modes: config.optional_modes.bits(),
            frames_in_flight: config.frames_in_flight,
            extension_count: raw_descs.len() as u32,
            extensions: raw_descs.as_mut_ptr(),
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

    pub fn submit(&mut self, queue: Queue, command_buffer: CommandBuffer) -> Result<()> {
        let result = unsafe {
            ffi::emgpu_device_submit_raw(
                &mut self.sys as *mut ffi::emgpu_device, 
                queue.0 as ffi::emgpu_queue, 
                command_buffer.data.as_ptr() as *const std::ffi::c_void,
                command_buffer.data.len() as u64,
                command_buffer.current_resource_idx) };

        if result != ffi::em_result_EMBER_RESULT_OK {
            return Err(result.into());
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub enum LoadOp {
    Load, Store, DontCore
}

#[derive(Clone, Copy)]
pub enum StoreOp {
    Store, DontCare
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Colour(pub u32);

impl Colour {
    pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(
            ((r as u32) << 24)
                | ((g as u32) << 16)
                | ((b as u32) << 8)
                | (a as u32),
        )
    }

    pub fn r(self) -> u8 {
        (self.0 >> 24) as u8
    }

    pub fn g(self) -> u8 {
        (self.0 >> 16) as u8
    }

    pub fn b(self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub fn a(self) -> u8 {
        self.0 as u8
    }
}

pub struct RenderpassColourAttachment {
    pub framebuffer: LocalFramebuffer,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
    pub colour: Colour,
    pub presentable: bool
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
    pub cmd_size: u64,
}
 
const HEADER_SIZE: usize = 16;
const _: () = assert!(size_of::<CommandHeader>() == HEADER_SIZE);
 
impl CommandHeader {
    fn write_to(&self, dst: &mut [u8]) {
        assert!(dst.len() >= HEADER_SIZE);
 
        dst[0..4].copy_from_slice(&(self.cmd_type as u32).to_le_bytes());
        dst[4..8].fill(0); // padding
        dst[8..16].copy_from_slice(&self.cmd_size.to_le_bytes());
    }
}
 
fn align_up(value: usize, alignment: usize) -> usize {
    debug_assert!(alignment.is_power_of_two());
 
    (value + alignment - 1) & !(alignment - 1)
}
 
// ---- Public handle / config types --------------------------------------------
 
#[derive(Clone, Copy)]
pub struct LocalResource(pub u32);
 
#[derive(Clone, Copy)]
pub struct LocalFramebuffer(pub u32);
 
pub struct RenderpassConfig<'a> {
    pub render_origin: [u32; 2],
    pub render_size: [u32; 2],
    pub colour_attachments: &'a [RenderpassColourAttachment],
}

impl Encode for RenderpassColourAttachment {
    const SIZE: usize = 20; // must equal sizeof(emgpu_colour_attachment)

    fn encode(&self, w: &mut Writer<'_>) {
        w.u32(self.framebuffer.0)
            .u32(self.load_op as u32)
            .u32(self.store_op as u32)
            .u8(self.colour.r())
            .u8(self.colour.g())
            .u8(self.colour.b())
            .u8(self.colour.a())
            .u8(self.presentable as u8)
            .pad(3); // tail padding
    }
}

/*
pub struct ComputepassConfig<'a> {
    pub pipeline: &'a Pipeline,
    pub import_resources: &'a [ResourceImport],
    pub export_resources: &'a [ResourceExport],
}
 
pub struct RasterBindInfo<'a> {
    pub pipeline: &'a Pipeline,
    pub import_resources: &'a [ResourceImport],
    pub export_resources: &'a [ResourceExport],
}
*/
 
// ---- Tiny little-endian payload writer ---------------------------------------
 
struct Writer<'a> {
    buf: &'a mut [u8],
    pos: usize,
}
 
impl Writer<'_> {
    fn u8(&mut self, v: u8) -> &mut Self {
        self.buf[self.pos] = v;
        self.pos += 1;
        self
    }

    fn u32(&mut self, v: u32) -> &mut Self {
        self.buf[self.pos..self.pos + 4].copy_from_slice(&v.to_le_bytes());
        self.pos += 4;
        self
    }
 
    fn u64(&mut self, v: u64) -> &mut Self {
        self.buf[self.pos..self.pos + 8].copy_from_slice(&v.to_le_bytes());
        self.pos += 8;
        self
    }
 
    fn f32(&mut self, v: f32) -> &mut Self {
        self.buf[self.pos..self.pos + 4].copy_from_slice(&v.to_le_bytes());
        self.pos += 4;
        self
    }
 
    fn uvec2(&mut self, v: [u32; 2]) -> &mut Self {
        self.u32(v[0]).u32(v[1])
    }
 
    fn uvec3(&mut self, v: [u32; 3]) -> &mut Self {
        self.u32(v[0]).u32(v[1]).u32(v[2])
    }
 
    /// Struct tail padding (bytes are already zeroed by `alloc`).
    fn pad(&mut self, n: usize) -> &mut Self {
        self.pos += n;
        self
    }
}
 
/// Pointer stored as raw bits, matching the C union layout.
/// The consumer assumes the referent outlives submission.
fn ptr_bits<T>(r: &T) -> u64 {
    r as *const T as usize as u64
}
 
// ---- CommandBuffer -----------------------------------------------------------
 
pub struct CommandBuffer {
    data: Vec<u8>,
    current_resource_idx: u32,
}

trait Encode {
    const SIZE: usize;
    fn encode(&self, w: &mut Writer<'_>);
}
 
impl CommandBuffer {
    pub fn create(_device: &Device) -> Result<CommandBuffer> {
        Ok(Self {
            data: Vec::with_capacity(4),
            current_resource_idx: 0,
        })
    }
 
    /// Reserve a header + `payload_size` bytes (16-byte aligned) and return the payload slice.
    fn alloc(&mut self, ty: CommandType, payload_size: usize) -> &mut [u8] {
        let offset = align_up(self.data.len(), 16);
        let total_size = HEADER_SIZE + payload_size;
        let end = offset + total_size;
 
        // `end` is always > len, so this always grows. New bytes (including any
        // alignment gap before `offset`) are zeroed.
        self.data.resize(end, 0);
 
        let header = CommandHeader {
            cmd_type: ty,
            cmd_size: total_size as u64,
        };
        header.write_to(&mut self.data[offset..offset + HEADER_SIZE]);
 
        &mut self.data[offset + HEADER_SIZE..end]
    }
 
    /// Allocate a fixed-size payload (N bytes) and fill it via `f`.
    fn push<const N: usize>(&mut self, ty: CommandType, f: impl FnOnce(&mut Writer<'_>)) {
        let mut w = Writer { buf: self.alloc(ty, N), pos: 0 };
        f(&mut w);
        debug_assert_eq!(w.pos, N, "payload layout mismatch for command {}", ty as u32);
    }
 
    /// Allocate an array payload and copy the elements in verbatim.
    fn push_array<T: bytemuck::NoUninit>(&mut self, ty: CommandType, items: &[T]) {
        let bytes: &[u8] = bytemuck::cast_slice(items);
        self.alloc(ty, bytes.len()).copy_from_slice(bytes);
    }

    fn push_encoded<T: Encode>(&mut self, ty: CommandType, items: &[T]) {
        let mut w = Writer { buf: self.alloc(ty, T::SIZE * items.len()), pos: 0 };
        for item in items {
            item.encode(&mut w);
        }
        debug_assert_eq!(w.pos, T::SIZE * items.len(), "layout mismatch for command {}", ty as u32);
    }
 
    fn next_resource_idx(&mut self) -> u32 {
        let idx = self.current_resource_idx;
        self.current_resource_idx += 1;
        idx
    }
 
    // ---- resources ----
 
    pub fn empty_resource(&mut self) -> LocalResource {
        let id = self.next_resource_idx();
        self.push::<4>(CommandType::EmptyResource, |w| {
            w.u32(id);
        });
        LocalResource(id)
    }
 
    pub fn import_texture(&mut self, texture: &Texture) -> LocalFramebuffer {
        let id = self.next_resource_idx();
        self.push::<16>(CommandType::ImportTexture, |w| {
            w.u64(ptr_bits(texture)).u32(id).pad(4); // C struct is 16 bytes (tail padding)
        });
        LocalFramebuffer(id)
    }
 
    pub fn acquire_surface(&mut self, surface: &Surface) -> LocalFramebuffer {
        let id = self.next_resource_idx();
        self.push::<16>(CommandType::AcquireSurface, |w| {
            w.u64(ptr_bits(surface)).u32(id).pad(4);
        });
        LocalFramebuffer(id)
    }
 
    // ---- compute ----
/*
    pub fn begin_computepass(&mut self, config: &ComputepassConfig) {
        self.push::<8>(CommandType::BeginComputePass, |w| {
            w.u64(ptr_bits(config.pipeline));
        });
        self.push_array(CommandType::ImportResources, config.import_resources);
        self.push_array(CommandType::ExportResources, config.export_resources);
    }
*/
    pub fn dispatch(&mut self, group_size: [u32; 3]) {
        self.push::<12>(CommandType::Dispatch, |w| {
            w.uvec3(group_size);
        });
    }
 
    pub fn end_computepass(&mut self) {
        self.alloc(CommandType::EndComputePass, 0);
    }
 
    // ---- graphics ----
 
    pub fn begin_renderpass(&mut self, config: &RenderpassConfig) {
        self.push::<16>(CommandType::BeginRenderPass, |w| {
            w.uvec2(config.render_origin).uvec2(config.render_size);
        });
        self.push_encoded(CommandType::ColourAttachments, config.colour_attachments);
    }
 
    pub fn end_renderpass(&mut self) {
        self.alloc(CommandType::EndRenderPass, 0);
    }
 
    pub fn set_viewport(&mut self, origin: [u32; 2], size: [u32; 2], min_depth: f32, max_depth: f32) {
        self.push::<24>(CommandType::SetViewport, |w| {
            w.uvec2(origin).uvec2(size).f32(min_depth).f32(max_depth);
        });
    }
 
    pub fn set_scissor(&mut self, origin: [u32; 2], size: [u32; 2]) {
        self.push::<16>(CommandType::SetScissor, |w| {
            w.uvec2(origin).uvec2(size);
        });
    }
 
    // ---- raster ----
/*
    pub fn bind_raster_pipeline(&mut self, info: &RasterBindInfo) {
        self.push::<8>(CommandType::BindRasterPipeline, |w| {
            w.u64(ptr_bits(info.pipeline));
        });
        self.push_array(CommandType::ImportResources, info.import_resources);
        self.push_array(CommandType::ExportResources, info.export_resources);
    }
 
    pub fn bind_vertex_buffers(&mut self, vertex_buffers: &[Buffer]) {
        // Buffer must be a NoUninit handle type, same as the C memcpy of emgpu_buffer.
        self.push_array(CommandType::BindVertexBuffers, vertex_buffers);
    }
 
    pub fn bind_index_buffer(&mut self, index_buffer: &Buffer) {
        self.push::<8>(CommandType::BindIndexBuffer, |w| {
            w.u64(ptr_bits(index_buffer));
        });
    }
*/
    pub fn draw(&mut self, vertex_count: u32, instance_count: u32) {
        self.push::<8>(CommandType::Draw, |w| {
            w.u32(vertex_count).u32(instance_count);
        });
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
    use crate::core::{Allocator, Error, Result, Version};
    use crate::gpu::{Device, Extension, GpuFormat, Surface, TextureUsage};
    use crate::window::{Window, Desktop};
    use crate::ffi;

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

    #[repr(C)]
    pub struct EmwinSurfaceParams {
        pub desktop: *const ffi::emwin_desktop,
        pub out_extension: *mut ffi::emgpu_emwin_surface_ext,
    }

    pub struct EmwinSurfaceExt {
        pub sys: ffi::emgpu_extension_desc,
        pub data: ffi::emgpu_emwin_surface_ext,
        desktop: *const ffi::emwin_desktop,
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
                desktop: desktop.sys as *const ffi::emwin_desktop,
            };

            ext.prepare();
            ext
        }

        fn write_params(&self) {
            let params = EmwinSurfaceParams {
                desktop: self.desktop,
                out_extension: &self.data as *const _ as *mut _,
            };

            assert_eq!(std::mem::size_of::<EmwinSurfaceParams>(), 16);

            unsafe {
                std::ptr::copy_nonoverlapping(
                    &params as *const EmwinSurfaceParams as *const u8,
                    self.sys.user_data.bytes.as_ptr() as *mut u8,
                    std::mem::size_of::<EmwinSurfaceParams>(),
                );
            }
        }

        pub fn create_surface(
            &self,
            device: &mut Device,
            allocator: &Allocator,
            config: &EmwinSurfaceConfig,
        ) -> Result<Surface> {
            let create_surface = self.data.create_surface.ok_or_else(|| {
                Error::UnavailableAPI
            })?;

            let window = config.window.as_ref().ok_or_else(|| {
                // Replace with your crate's actual missing-window error.
                Error::InvalidValue
            })?;

            let mut c_config = ffi::emgpu_emwin_surface_config {
                preferred_format: config.preferred_format.0.0,
                force_format: config.force_format,
                min_texture_count: config.min_texture_count,
                usage: config.usage.bits(),
                window: &window.sys as *const ffi::emwin_window,
            };

            let mut surface = Surface {
                sys: unsafe { std::mem::zeroed() },
            };

            let result = unsafe {
                create_surface(
                    &mut device.sys,
                    allocator.sys.get() as *mut ffi::em_allocator,
                    &mut c_config,
                    &mut surface.sys,
                )
            };

            if result != ffi::em_result_EMBER_RESULT_OK {
                return Err(result.into());
            }

            Ok(surface)
        }
    }

    impl Extension for EmwinSurfaceExt {
        fn desc(&self) -> &ffi::emgpu_extension_desc {
            &self.sys
        }

        fn prepare(&self) {
            self.write_params();
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
