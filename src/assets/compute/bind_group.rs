use crate::assets::compute::pipeline::ComputePipeline;
use crate::assets::{Texture};
use crate::assets::buffer::Buffer;
use crate::assets_manager::Handle;
use crate::Renderer;

pub struct ComputeBindGroup{
    pub bind_group: wgpu::BindGroup
}

pub struct ComputeBindGroupBuilder{
    textures: Vec<(u32, Handle<Texture>)>,
    storages: Vec<(u32,Handle<Buffer>)>,
    pipeline: Handle<ComputePipeline>,
}

impl ComputeBindGroupBuilder{
    pub fn new(pipeline: Handle<ComputePipeline>) -> Self{
        Self{
            textures: vec![],
            storages: vec![],
            pipeline,
        }
    }

    pub fn storage_texture(
        mut self,
        texture_binding: u32,
        texture: Handle<Texture>,
    ) -> Self {
        self.textures.push((texture_binding, texture));
        self
    }
    pub fn storage(
        mut self,
        binding: u32,
        buffer: Handle<Buffer>,
    ) -> Self {
        self.storages.push((binding, buffer));
        self
    }

    pub fn build(&self,renderer: &mut Renderer) -> Handle<ComputeBindGroup> {
        let pipeline = renderer.asset_manager.compute_pipelines.get(self.pipeline).unwrap();

        let mut entries = Vec::new();

        for (binding,handle) in &self.storages {
            let storage = renderer.asset_manager.buffers.get(*handle).unwrap();

            entries.push(wgpu::BindGroupEntry {
                binding:*binding,
                resource: storage.buffer.as_entire_binding(),
            });
        }

        for (tex_binding,handle) in &self.textures {
            let texture = renderer.asset_manager.textures.get(*handle).unwrap();

            entries.push(wgpu::BindGroupEntry {
                binding: *tex_binding,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            });
        }

        let bind_group = renderer.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("material bind group"),
                layout: &pipeline.layout,
                entries: &entries,
            }
        );

        renderer.asset_manager.compute_bind_groups.insert(ComputeBindGroup{
            bind_group,
        })
    }
}