use std::ops::Range;
use crate::assets::buffer::Buffer;
use crate::assets::compute::bind_group::ComputeBindGroup;
use crate::assets::compute::pipeline::ComputePipeline;
use crate::assets::compute::task::ComputeTask;
use crate::assets_manager::handle::Handle;
use crate::assets::render::material::Material;
use crate::assets::render::mesh::Mesh;

pub(crate) struct RenderItem {
    pub mesh: Handle<Mesh>,
    pub material: Handle<Material>,
    pub instances: Option<Handle<Buffer>>,
    pub range: Option<Range<u32>>
}

impl RenderItem {
    fn new(mesh: Handle<Mesh>, material: Handle<Material>,instances: Option<Handle<Buffer>>,range: Option<Range<u32>>) -> Self {
        Self{
            mesh,
            material,
            instances,
            range,
        }
    }
}

pub struct Frame {
    pub(crate) render_tasks: Vec<RenderItem>,
    pub(crate) compute_tasks: Vec<ComputeTask>,
}

impl Frame {
    pub(crate) fn new() -> Self {
        Self {
            render_tasks: Vec::new(),
            compute_tasks: Vec::new(),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.render_tasks.clear();
        self.compute_tasks.clear();
    }

    pub fn draw_mesh(&mut self,material: Handle<Material>,mesh: Handle<Mesh>) {
        let item = RenderItem::new(mesh, material,None,None);
        self.render_tasks.push(item);
    }

    pub fn draw_instances(&mut self,instances: Handle<Buffer>,material: Handle<Material>,mesh: Handle<Mesh>,range: Range<u32>) {
        let item = RenderItem::new(mesh, material, Some(instances), Some(range));
        self.render_tasks.push(item);
    }

    pub fn compute(&mut self,bind_group: Handle<ComputeBindGroup>, pipeline: Handle<ComputePipeline>, dispatch: (u32,u32,u32)){
        let task = ComputeTask::new(pipeline,bind_group,dispatch);
        self.compute_tasks.push(task);
    }

    pub fn sort_by_material(&mut self) {
        self.render_tasks.sort_by_key(|item| {
            item.material.index
        });
    }
    pub fn sort_by_mesh(&mut self) {
        self.render_tasks.sort_by_key(|item| {
            item.mesh.index
        });
    }
    pub fn sort(&mut self) {
        self.render_tasks.sort_by_key(|item| {
            (
                item.material.index,
                item.mesh.index,
            )
        });
    }
}