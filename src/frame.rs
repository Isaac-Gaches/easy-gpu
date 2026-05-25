use std::ops::Range;
use bytemuck::bytes_of;
use crate::assets::buffer::Buffer;
use crate::assets::compute::bind_group::ComputeBindGroup;
use crate::assets::compute::pipeline::ComputePipeline;
use crate::assets::compute::task::ComputeTask;
use crate::assets_manager::handle::Handle;
use crate::assets::render::material::Material;
use crate::assets::render::mesh::Mesh;
use crate::assets::{GpuInstance, Texture};
use crate::assets::render::task::{DrawCommand, InstancedCommand, RenderTask, StreamedCommand};

pub struct Frame {
    pub(crate) render_tasks: Vec<RenderTask>,
    pub(crate) compute_tasks: Vec<ComputeTask>,
    pub(crate) textures_to_clear: Vec<Handle<Texture>>,
    pub(crate) instance_bytes: Vec<u8>,
}

impl Frame {
    pub(crate) fn new() -> Self {
        Self {
            render_tasks: Vec::new(),
            compute_tasks: Vec::new(),
            textures_to_clear: Vec::new(),
            instance_bytes: Vec::new(),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.render_tasks.clear();
        self.compute_tasks.clear();
    }

    pub fn draw(&mut self,material: Handle<Material>,mesh: Handle<Mesh>) {
        let item = RenderTask::Draw(DrawCommand{
            mesh,
            material,
        });
        self.render_tasks.push(item);
    }

    pub fn draw_manual_batch(&mut self,instances: Vec<Handle<Buffer>>,material: Handle<Material>,mesh: Handle<Mesh>,range: Range<u32>) {
        let item = RenderTask::DrawStreamed(StreamedCommand{
            mesh,
            material,
            instances,
            range,
        });
        self.render_tasks.push(item);
    }

    pub fn draw_batch<T: GpuInstance>(&mut self,instance: &[T],material: Handle<Material>,mesh: Handle<Mesh>){
        let instance_count = instance.len() as u32;
        let start = self.instance_bytes.len();
        self.instance_bytes.extend_from_slice(bytemuck::cast_slice(&instance));
        let end = self.instance_bytes.len();
        let instance_range = start as u64..end as u64;

        let item = RenderTask::DrawInstanced(InstancedCommand{
            mesh,
            material,
            instance_range,
            instance_count,
        });

        self.render_tasks.push(item);    }

    pub fn compute(&mut self,bind_group: Handle<ComputeBindGroup>, pipeline: Handle<ComputePipeline>, dispatch: (u32,u32,u32)){
        let task = ComputeTask::new(pipeline,bind_group,dispatch);
        self.compute_tasks.push(task);
    }

    pub fn sort_by_material(&mut self) {
        self.render_tasks.sort_by_key(|item| {
            match item {
                RenderTask::Draw(cmd) =>{
                    cmd.material.index
                }
                RenderTask::DrawInstanced(cmd) =>{
                    cmd.material.index
                }
                RenderTask::DrawStreamed(cmd) =>{
                    cmd.material.index
                }
            }
        });
    }
    pub fn sort_by_mesh(&mut self) {
        self.render_tasks.sort_by_key(|item| {
            match item {
                RenderTask::Draw(cmd) =>{
                    cmd.mesh.index
                }
                RenderTask::DrawInstanced(cmd) =>{
                    cmd.mesh.index
                }
                RenderTask::DrawStreamed(cmd) =>{
                    cmd.mesh.index
                }
            }
        });
    }
    pub fn sort(&mut self) {
        self.render_tasks.sort_by_key(|item| {
            match item {
                RenderTask::Draw(cmd) =>{
                   (cmd.material.index,
                    cmd.mesh.index)
                }
                RenderTask::DrawInstanced(cmd) =>{
                    (cmd.material.index,
                     cmd.mesh.index)
                }
                RenderTask::DrawStreamed(cmd) =>{
                    (cmd.material.index,
                     cmd.mesh.index)
                }
            }
        });
    }

    pub fn request_texture_clear(&mut self, texture: Handle<Texture>){
        self.textures_to_clear.push(texture);
    }
}