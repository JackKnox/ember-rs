use crate::core::{Allocator, Error, Result, Version};
use ffi;

use std::ffi::CString;

use bitflags::bitflags;

////////////////////////////////////////////////

pub enum DescriptorType {
    StorageBuffer,
    StorageImage,
    UniformBuffer,
    SampledImage,
}

pub enum ShaderStageType {
    Vertex,
    Fragment,
    Compute,
}

pub enum BlendFactor {
    Zero,
    One,
    SrcColour,
    OneMinusSrcColour,
    DstColour,
    OneMinusDstColour,
    DstAlpha,
    OneMinusDstAlpha,
    ConstantColour,
    OneMinusConstantColour,
    ConstantAlpha,
    OneMinusConstantAlpha,
}

pub enum BlendOp {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

pub enum CompareOp {
    Never,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
    Always,
}

pub enum StencilOp {
    Keep,
    Zero,
    Replace,
    IncrementAndClamp,
    DecrementAndClamp,
    Invert,
    IncrementAndWrap,
    DecrementAndWrap,
}

pub enum PrimitiveType {
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
}

pub struct ShaderSrc<'a> {
    pub data: &'a [u32],
    pub entry_point: &'static str,
}

impl Into<ffi::emgpu_shader_src> for ShaderSrc<'_> {
    fn into(self) -> ffi::emgpu_shader_src {
        ffi::emgpu_shader_src {
            data: self.data.as_ptr(),
            size: (self.data.len() * size_of::<u32>()) as u64,
            entry_point: self.entry_point.as_ptr() as *const i8,
        }
    }
}

pub struct DescriptorDesc {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
    pub stage_type: ShaderStageType,
}

pub struct PipelineColourAttachment {
    pub format: GpuFormat,
    pub blend_enable: bool,
    pub src_colour: BlendFactor,
    pub dst_colour: BlendFactor,
    pub colour_op: BlendOp,
    pub src_alpha: BlendFactor,
    pub dst_alpha: BlendFactor,
}

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
    pub vendor_signiture: VendorSigniture,
}

////////////////////////////////////////////////

pub struct LocalResource(ffi::emgpu_local_resource);

pub struct LocalFramebuffer(ffi::emgpu_local_framebuffer);

pub struct CommandBuffer {
    sys: ffi::emgpu_command_buffer,
}

pub struct ComputeEncoder {}

pub struct RenderEncoder {}

impl CommandBuffer {
    pub fn create(device: &mut Device) -> Result<CommandBuffer> {
        let mut command_buffer = CommandBuffer {
            sys: unsafe { std::mem::zeroed() },
        };
        let result = unsafe {
            ffi::emgpu_command_buffer_create(
                &mut device.sys as *mut ffi::emgpu_device,
                &mut command_buffer.sys as *mut ffi::emgpu_command_buffer,
            )
        };

        if result != ffi::em_result_EMBER_RESULT_OK {
            return Err(result.into());
        }

        Ok(command_buffer)
    }

    pub fn empty_resource(&mut self) -> LocalResource {}

    pub fn import_texture(&mut self, texture: &mut Texture) -> LocalFramebuffer {}

    pub fn acquire_surface(&mut self, surface: &mut Surface) -> LocalFramebuffer {}

    pub fn begin_computepass(&mut self, config: &ComputePassConfig) -> ComputeEncoder {}

    pub fn begin_renderpass(&mut self, config: &RenderPassConfig) -> RenderEncoder {}
}

impl ComputeEncoder {
    pub fn dispatch(&mut self, group_size: [u32; 3]) {}

    pub fn end(&mut self) {}
}

impl Drop for ComputeEncoder {
    fn drop(&mut self) {
        self.end();
    }
}

impl RenderEncoder {
    pub fn bind_raster_pipeline(&mut self, pipeline: RasterBindInfo) {}

    pub fn draw(&mut self, vertex_count: u32, instance_count: u32) {}

    pub fn end(&mut self) {}
}

impl Drop for RenderEncoder {
    fn drop(&mut self) {
        self.end();
    }
}

pub struct RasterDepthStencilConfig {
    pub depth_format: GpuFormat,
    pub stencil_format: GpuFormat,
    pub depth_test_enable: bool,
    pub depth_write_enable: bool,
    pub compare_op: CompareOp,
    pub depth_bounds_test_enabled: bool,
    pub stencil_test_enable: bool,
    pub front_stencil: StencilOp,
    pub back_stencil: StencilOp,
    pub min_depth_bounds: f32,
    pub max_depth_bounds: f32,
}

pub struct RasterVertexConfig {
    pub topology: PrimitiveType,
    pub attributes: Vec<GpuFormat>,
}

pub struct RasterPipelineConfig<'a> {
    vertex_shader: ShaderSrc<'a>,
    fragment_shader: ShaderSrc<'a>,
    descriptors: Vec<DescriptorDesc>,
    colour_attachments: Vec<PipelineColourAttachment>,
    depth_stencil: Option<RasterDepthStencilConfig>,
    vertex_input: Option<RasterVertexConfig>,
}

pub struct ComputePipelineConfig<'a> {
    shader: ShaderSrc<'a>,
    descriptors: Vec<DescriptorDesc>,
}

pub struct Pipeline {
    sys: ffi::emgpu_pipeline,
}

impl Pipeline {
    pub fn create_raster(
        device: &mut Device,
        allocator: &Allocator,
        config: &RasterPipelineConfig,
    ) -> Result<Pipeline> {
        let c_config = ffi::emgpu_raster_pipeline_config {
            api_next: std::ptr::null_mut(),
            vertex_shader: config.vertex_shader.into(),
            fragment_shader: config.fragment_shader.into(),
            descriptor_count: config.descriptors.len() as u32,
            descriptors: config.descriptors.as_mut_ptr(),
            colour_attachment_count: config.colour_attachments.len() as u32,
            colour_attachments: config.colour_attachments.as_mut_ptr(),
            depth_stencil: match config.depth_stencil {
                None => std::ptr::null_mut(),
                Some(conf) => conf.into(),
            },
            vertex_input: match config.vertex_input {
                None => std::ptr::null_mut(),
                Some(conf) => conf.into(),
            },
        };

        let mut pipline = Pipeline {
            sys: unsafe { std::mem::zeroed() },
        };

        let result = unsafe {
            ffi::emgpu_raster_pipeline_create(
                &mut device.sys as *mut ffi::emgpu_device,
                &mut allocator.sys as *mut ffi::em_allocator,
                &mut c_config as *mut ffi::emgpu_raster_pipeline_config,
                &mut pipline.sys as *mut ffi::emgpu_pipeline,
            )
        };

        if result != ffi::em_result_EMBER_RESULT_OK {
            return Err(result.into());
        }

        Ok(pipline)
    }

    pub fn create_compute(
        device: &mut Device,
        allocator: &Allocator,
        config: &ComputePipelineConfig,
    ) -> Result<Pipeline> {
        let c_config = ffi::emgpu_compute_pipeline_config {
            api_next: std::ptr::null_mut(),
            shader: config.shader.into(),
            descriptor_count: config.descriptors.len() as u32,
            descriptors: config.descriptors.as_mut_ptr(),
        };

        let mut pipline = Pipeline {
            sys: unsafe { std::mem::zeroed() },
        };

        let result = unsafe {
            ffi::emgpu_compute_pipeline_create(
                &mut device.sys as *mut ffi::emgpu_device,
                &mut allocator.sys as *mut ffi::em_allocator,
                &mut c_config as *mut ffi::emgpu_compute_pipeline_config,
                &mut pipline.sys as *mut ffi::emgpu_pipeline,
            )
        };

        if result != ffi::em_result_EMBER_RESULT_OK {
            return Err(result.into());
        }

        Ok(pipline)
    }

    pub fn upload_descriptors(
        &mut self,
        device: &mut Device,
        descriptors: Vec<UpdateDescriptors>,
    ) -> Result<()> {
    }

    pub fn destroy(&mut self, device: &mut Device, allocator: &Allocator) {
        unsafe {
            ffi::emgpu_pipeline_destroy(
                &mut device.sys as *mut ffi::emgpu_device,
                &mut allocator.sys as *mut ffi::em_allocator,
                &mut self.sys as *mut ffi::emgpu_pipeline,
            )
        }
    }
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

impl Device {
    pub fn init(allocator: &Allocator, config: &DeviceConfig) -> Result<Device> {}

    pub fn capabilities(&mut self) -> Result<DeviceCapabilities> {}

    pub fn open_queue(&mut self) -> Result<Queue> {}

    pub fn wait_idle(&mut self, queue: Queue) -> Result<()> {}

    pub fn shutdown(&mut self, allocator: &Allocator) {}
}

/*

pub enum DeviceType {
    Other = 0,
    Intergrated = 1,
    Discrete = 2,
    Virtual = 3,
    CPU = 4,
}

pub struct DeviceVendorId(u32);

trait Extension {}

pub struct DeviceConfig<'a> {
    pub debug_name: &'a str,
    pub frame_allocator: Allocator,
    pub app_version: Version,
    pub required_modes: DeviceMode,
    pub optional_modes: DeviceMode,
    pub frames_in_flight: u32,
    pub extension: Vec<Box<dyn Extension>>,
}

pub struct DeviceCapabilities {
    pub device_name: String,
    pub device_type: DeviceType,
    pub enabled_modes: DeviceMode,
    pub driver_version: Version,
    pub max_anisotropy: f32,
    pub vendor_id: DeviceVendorId,
}

pub struct Device {
    sys: ffi::emgpu_device,
}

impl Device {
    pub fn init(allocator: &Allocator, config: &DeviceConfig) -> Result<Device> {
        let debug_name =
            CString::new(config.debug_name).expect("Device debug name contained a NULL byte");

        let c_config = ffi::emgpu_device_config {
            api_next: std::ptr::null_mut(),
            debug_name: debug_name.as_ptr(),
            frame_allocator: config.frame_allocator.into(),
            app_version: config.app_version.into(),
            required_modes: config.required_modes.bits(),
            optional_modes: config.optional_modes.bits(),
            frames_in_flight: config.frames_in_flight,
        };
        let mut device = Device {
            sys: unsafe { std::mem::zeroed() },
        };
        let result = unsafe {
            ffi::emgpu_device_init(
                &c_config as *const ffi::emgpu_device_config,
                allocator.into(),
                &mut device.sys as *mut ffi::emgpu_device
            )
        };

        if result != ffi::em_result_EMBER_RESULT_OK {
            return Err(result.into());
        }

        Ok(device)
    }

    pub fn shutdown(&mut self, allocator: &Allocator) {
        todo!()
    }

    pub fn capabilities(&self) -> DeviceCapabilities {
        todo!()
    }

    pub fn submit(&mut self, command_buffer: CommandBuffer) -> Result<()> {
        todo!()
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BufferUsage: u32 {
        const Vertex            = 0b00000001;
        const Index             = 0b00000010;
        const Uniform           = 0b00000100;
        const Storage           = 0b00001000;
        const TransferSrc       = 0b00010000;
        const TransferDst       = 0b00100000;
        const CpuVisible        = 0b01000000;
    }
}

pub struct BufferConfig {
    pub size: u64,
    pub usage: BufferUsage,
}

pub struct Buffer {
    sys: ffi::emgpu_buffer,
}

impl Buffer {
    pub fn create(device: &Device, allocator: &Allocator, config: &BufferConfig) -> Result<Buffer> {
        let c_config = ffi::emgpu_buffer_config {
            api_next: std::ptr::null_mut(),
            buffer_size: config.size,
            usage: config.usage.bits(),
        };

        let mut buffer = Buffer {
            sys: unsafe { std::mem::zeroed() },
        };
        let result = unsafe {
            ffi::emgpu_buffer_create(
                &device.sys as *const ffi::emgpu_device,
                allocator.into(),
                &c_config as *const ffi::emgpu_buffer_config,
                &mut buffer.sys as *mut ffi::emgpu_buffer,
            )
        };
    }

    pub fn destroy(&mut self, device: &Device, allocator: &Allocator) {
        todo!()
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct TextureUsage: u32 {
        const Storage            = 0b00000001;
        const Sampled            = 0b00000010;
        const TransferSrc        = 0b00000100;
        const TransferDst        = 0b00001000;
        const AttachmentDst      = 0b00010000;
    }
}

pub enum FilterType {
    Nearest,
    Linear,
}

pub enum AddressMode {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
    MirrorClampToEdge,
}

pub struct TextureConfig {
    pub format: GpuFormat,
    pub filter_type: FilterType,
    pub address_mode: AddressMode,
    pub usage: TextureUsage,
    pub size: [u32; 2],
    pub max_anisotropy: f32,
}

pub struct Texture {
    sys: ffi::emgpu_texture,
}

impl Texture {
    pub fn create(
        device: &Device,
        allocator: &Allocator,
        config: &TextureConfig,
    ) -> Result<Texture> {
        todo!()
    }

    pub fn destroy(&mut self, device: &Device, allocator: &Allocator) {
        todo!()
    }

    pub fn size_in_bytes(&self) -> u64 {}
}

pub struct Surface {
    sys: ffi::emgpu_surface,
}

impl Surface {
    pub fn resize(&mut self, device: &Device, new_size: [u32; 2]) -> Result<()> {
        todo!()
    }

    pub fn destroy(&mut self, device: &Device, allocator: &Allocator) {
        todo!()
    }
}


pub enum BlendFactor {
    Zero,
    One,
    SrcColour,
    OneMinusSrcColour,
    DstColour,
    OneMinusDstColour,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
    ConstantColour,
    OneMinusConstantColour,
    ConstantAlpha,
    OneMinusConstantAlpha,
}

pub enum BlendOp {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

pub enum PrimitiveType {
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
}

pub struct ShaderSource<'a> {
    pub data: &'a [u32],
    pub entry_point: &'static str,
}

pub struct DescriptorDesc {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
    pub stage_type: ShaderStageType,
}

pub struct RasterBlendConfig {
    pub src_colour: BlendFactor,
    pub dst_colour: BlendFactor,
    pub colour_op: BlendOp,
    pub src_alpha: BlendFactor,
    pub dst_alpha: BlendFactor,
    pub alpha_op: BlendOp,
}

pub struct RasterVertexInputConfig {
    pub topology: PrimitiveType,
    pub attributes: Vec<GpuFormat>,
}

pub struct RasterPiplineConfig<'a> {
    vertex_shader: ShaderSource<'a>,
    fragment_shader: Option<ShaderSource<'a>>,
    descriptors: Vec<DescriptorDesc>,
    blend_state: Option<RasterBlendConfig>,
    vertex_input: Option<RasterVertexInputConfig>,
}

pub struct Pipeline {
    sys: ffi::emgpu_pipeline,
}

impl Pipeline {
    pub fn create_raster(
        device: &Device,
        allocator: &Allocator,
        config: RasterPiplineConfig,
    ) -> Result<Pipeline> {
        todo!()
    }

    pub fn create_compute(
        device: &Device,
        allocator: &Allocator,
        config: ComputePiplineConfig,
    ) -> Result<Pipeline> {
        todo!()
    }

    pub fn destroy(&mut self, device: &Device, allocator: &Allocator) {
        todo!()
    }
}

struct CommandBuffer {
    sys: ffi::emgpu_frame,
}
*/

/*

impl CommandBuffer {
    pub fn create(device: &Device) -> Result<CommandBuffer> {
        todo!()
    }

    pub fn acquire_surface(&mut self, surface: &Surface) -> LocalTexture {

    }

    pub fn import_texture(&mut self, texture: &Texture) -> LocalTexture {

    }

    pub fn begin_renderpass(&mut self, renderpass_config: RenderPassConfig) -> RenderEncoder {

    }
}

struct RenderPassConfig {
    label: &String,
    colour_attachments: &[ColourAttachment],
}

struct RenderEncoder {

}

impl RenderEncoder {
    pub fn set_viewport(&mut self, origin: [u32; 2], size: [u32; 2], min_depth: f32, max_depth: f32) {

    }

    pub fn set_scissor(&mut self, origin: [u32; 2], size: [u32; 2]) {

    }

}
*/
