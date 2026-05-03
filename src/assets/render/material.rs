use wgpu::Device;
use crate::assets::buffer::Buffer;
use crate::assets_manager::handle::Handle;
use crate::assets::render::pipeline::RenderPipeline;
use crate::assets::texture::Texture;
use crate::assets_manager::asset_manager::AssetManager;

pub struct Material {
    pub bind_group: wgpu::BindGroup,
    pub pipeline: Handle<RenderPipeline>,
}

pub struct MaterialBuilder {
    textures: Vec<(u32,u32, Handle<Texture>)>,
    uniforms: Vec<(u32,Handle<Buffer>)>,
    pipeline: Handle<RenderPipeline>,
}

impl MaterialBuilder {
    pub fn new(
        pipeline: Handle<RenderPipeline>,
    ) -> Self {
        Self {
            textures: Vec::new(),
            uniforms: Vec::new(),
            pipeline,
        }
    }

    pub fn texture(
        mut self,
        texture_binding: u32,
        sampler_binding: u32,
        texture: Handle<Texture>,
    ) -> Self {
        self.textures.push((texture_binding,sampler_binding, texture));
        self
    }


    pub fn uniform(
        mut self,
        binding: u32,
        buffer: Handle<Buffer>,
    ) -> Self {
        self.uniforms.push((binding, buffer));
        self
    }

    pub fn build(self,device:&Device, asset_manager: &AssetManager) -> Material {
        let pipeline = asset_manager.render_pipelines.get(self.pipeline.clone()).unwrap();

        let mut entries = Vec::new();

        for (binding,handle) in self.uniforms {
            let uniform = asset_manager.buffers.get(handle).unwrap();
            entries.push(wgpu::BindGroupEntry {
                binding,
                resource: uniform.buffer.as_entire_binding(),
            });
        }

        for (tex_binding,sampler_binding,handle) in self.textures {
            let texture = asset_manager.textures.get(handle).unwrap();
        entries.push(wgpu::BindGroupEntry {
                 binding: tex_binding,
                 resource: wgpu::BindingResource::TextureView(&texture.view),
             });
            entries.push(wgpu::BindGroupEntry {
                binding: sampler_binding,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            });
        }

        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("material bind group"),
                layout: &pipeline.material_layout,
                entries: &entries,
            }
        );

        Material {
            bind_group,
            pipeline: self.pipeline,
        }
    }
}