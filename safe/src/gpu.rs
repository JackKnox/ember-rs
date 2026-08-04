use ffi;
use crate::core::{Allocator, Version, Result};

use bitflags::bitflags;

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
    pub extension: Vec<Box<dyn Extension>>
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
    sys: ffi::emgpu_device
}

impl Device {
    pub fn init(allocator: Allocator, config: &DeviceConfig) -> Result<Device> {
        todo!()
    }

    pub fn shutdown(&mut self, allocator: Allocator) {
        todo!()
    }

    pub fn capabilities(&self) -> DeviceCapabilities {
        todo!()
    }

    pub fn print_capabilities(&self) {
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
        const CpuVisible        = 0b00100000;
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
        todo!()
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
        const AttachmentDst      = 0b00001000;
    }
}

pub enum FilterType {
    Nearest, Linear,
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
    sys: ffi::emgpu_texture
}

impl Texture {
    pub fn create(device: &Device, allocator: &Allocator, config: &TextureConfig) -> Result<Texture> {
        todo!()
    }

    pub fn destroy(&mut self, device: &Device, allocator: &Allocator) {
        todo!()
    }

    pub fn size_in_bytes(&self) -> u64 {

    }
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

pub enum DescriptorType {
    StorageBuffer,
    StorageImage,
    UniformBuffer,
    SampledImage,
}

pub enum ShaderStageType {
    Vertex, Fragment, Compute
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
    Add, Subtract, ReverseSubtract, Min, Max,
}

pub enum PrimitiveType {
    PointList, LineList, LineStrip, TriangleList, TriangleStrip
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
    pub fn create_raster(device: &Device, allocator: &Allocator, config: RasterPiplineConfig) -> Result<Pipeline> {
        todo!()
    }

    pub fn create_compute(device: &Device, allocator: &Allocator, config: ComputePiplineConfig) -> Result<Pipeline> {
        todo!()
    }

    pub fn destroy(&mut self, device: &Device, allocator: &Allocator) {
        todo!()
    }
}

struct CommandBuffer {
    sys: ffi::emgpu_frame,
}

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
