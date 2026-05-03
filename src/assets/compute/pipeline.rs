use wgpu::{BindGroupLayout, BufferBindingType, ComputePipelineDescriptor, Device, ShaderModule, StorageTextureAccess, TextureFormat, TextureViewDimension};
use crate::assets_manager::asset_manager::AssetManager;
use crate::assets_manager::Handle;

pub struct ComputePipeline{
    pub pipeline: wgpu::ComputePipeline,
    pub layout: BindGroupLayout,
}

pub struct ComputePipelineBuilder{
    shader: Handle<ShaderModule>,
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl ComputePipelineBuilder{
    pub fn new(shader: Handle<ShaderModule>)->Self{
        Self{
            shader,
            entries: vec![],
        }
    }
    pub fn bind_group_layout(
        mut self,
        entries: &[wgpu::BindGroupLayoutEntry],
    ) -> Self {
        self.entries = entries.to_vec();
        self
    }
    pub(crate) fn build(self,device: &Device,asset_manager: &AssetManager) -> ComputePipeline{
        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("material layout"),
                entries: &self.entries,
            }
        );
        
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label: None,
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor{
            label: Some("Compute Pipeline"),
            layout: Some(&layout),
            module: asset_manager.shaders.get(self.shader).unwrap(),
            entry_point: Some("cs_main"),
            compilation_options: Default::default(),
            cache: None,
        });

        ComputePipeline{
            pipeline,
            layout: bind_group_layout,
        }
    }
}

pub fn storage(binding: u32,read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub fn storage_texture(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::StorageTexture {
            access: StorageTextureAccess::WriteOnly,
            format: TextureFormat::Rgba8Unorm,
            view_dimension: TextureViewDimension::D2,
        },
        count: None,
    }
}

/*pub fn texture(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}*/

