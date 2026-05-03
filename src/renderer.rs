use std::sync::Arc;
use wgpu::{BufferUsages, Device, Queue, ShaderModule, StoreOp, Surface, SurfaceConfiguration};
use winit::dpi::PhysicalSize;
use winit::window::Window;
use crate::{frame::Frame};
use crate::assets::buffer::Buffer;
use crate::assets::compute::bind_group::{ComputeBindGroup, ComputeBindGroupBuilder};
use crate::assets::compute::pipeline::{ComputePipeline, ComputePipelineBuilder};
use crate::assets::render::material::{Material, MaterialBuilder};
use crate::assets::render::mesh::Mesh;
use crate::assets::render::pipeline::{RenderPipeline, RenderPipelineBuilder};
use crate::assets::vertex_layout::GpuVertex;
use crate::assets_manager::asset_manager::AssetManager;
use crate::assets_manager::asset_registry::AssetRegistry;
use crate::assets_manager::handle::Handle;

pub struct Renderer {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,

    pub asset_registry: AssetRegistry,
    pub asset_manager: AssetManager,

    depth_texture: Option<wgpu::Texture>,
    depth_view: Option<wgpu::TextureView>,

    frame: Frame,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor::default(),
        ).await.unwrap();

        let caps = surface.get_capabilities(&adapter);

        let surface_config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        let asset_manager = AssetManager::new();
        let asset_registry = AssetRegistry::new();

        let frame = Frame::new();

        Self {
            device,
            queue,
            surface,
            surface_config,
            asset_registry,
            asset_manager,
            depth_texture: None,
            depth_view: None,
            frame,
        }
    }

    pub fn render(&self) {
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                // still usable, but should reconfigure soon
                frame
            }

            wgpu::CurrentSurfaceTexture::Timeout => {
                return; // skip frame
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                return; // window hidden
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                // reconfigure surface
                self.surface.configure(&self.device, &self.surface_config);
                return;
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                // recreate surface ideally, but reconfigure for now
                self.surface.configure(&self.device, &self.surface_config);
                return;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return;
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            },
        );

        for compute_task in &self.frame.compute_tasks{
            compute_task.execute(&mut encoder, &self.asset_manager)
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),

                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: match &self.depth_view{
                    Some(view) => {
                        Some(wgpu::RenderPassDepthStencilAttachment {
                            view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: StoreOp::Store,
                            }),
                            stencil_ops: None,
                        })
                    }
                    None => None
                } ,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            for item in &self.frame.render_tasks {
                let material = self.asset_manager.materials.get(item.material).unwrap();
                let pipeline = self.asset_manager.render_pipelines.get(material.pipeline).unwrap();
                let mesh = self.asset_manager.meshes.get(item.mesh).unwrap();
                let instances = self.asset_manager.buffers.get(item.instances).unwrap();

                render_pass.set_pipeline(&pipeline.pipeline);
                render_pass.set_bind_group(0, &material.bind_group, &[]);
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, instances.buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                render_pass.draw_indexed(0..mesh.index_count, 0,item.range.clone());
            }
        }

        self.queue.submit(Some(encoder.finish()));

        output.present();
    }

    pub fn resize_surface(&mut self, size: PhysicalSize<u32>) {
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);
        self.create_depth_texture(size.width, size.height);
    }
    
    pub fn window_aspect(&self) -> f32 {
        self.surface_config.width as f32 / self.surface_config.height as f32
    }

    pub fn begin_frame(&mut self) -> &mut Frame {
        self.frame.clear();
        &mut self.frame
    }

    pub fn current_frame(&mut self) -> &mut Frame {
        &mut self.frame
    }

    pub fn create_mesh<T: GpuVertex>(&mut self, vertices: &[T],indices: &[u32]) -> Handle<Mesh>{
        let mesh = Mesh::new(&self.device,vertices,indices);
        self.asset_manager.meshes.insert(mesh)
    }

    pub fn create_render_pipeline(&mut self, builder: RenderPipelineBuilder) -> Handle<RenderPipeline> {
        if builder.depth_format.is_some() && self.depth_texture.is_none() {
            self.create_depth_texture(self.surface_config.width,self.surface_config.height);
        }

        let pipeline = builder.build(&self.device,&self.asset_manager,&self.surface_config);
        self.asset_manager.render_pipelines.insert(pipeline)
    }

    fn create_depth_texture(&mut self, width: u32, height: u32){
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("depth_texture"),
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.depth_view = Some(view);
        self.depth_texture = Some(texture);
    }

    pub fn create_material(&mut self,builder: MaterialBuilder) -> Handle<Material> {
        let material = builder.build(&self.device,&self.asset_manager);
        self.asset_manager.materials.insert(material)
    }

    pub fn create_buffer(&mut self,buffer_usages: BufferUsages,size:u64) -> Handle<Buffer> {
        let buffer = Buffer::new(&self.device,size,buffer_usages);
        self.asset_manager.buffers.insert(buffer)
    }

    pub fn create_buffer_with_contents(&mut self,buffer_usages: BufferUsages,contents:&[u8]) -> Handle<Buffer> {
        let buffer = Buffer::from_contents(&self.device,contents,buffer_usages);
        self.asset_manager.buffers.insert(buffer)
    }

    pub fn load_shader(&mut self,src: &'static str) -> Handle<ShaderModule>{
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(src.into()),
        });
        self.asset_manager.shaders.insert(shader)
    }

    pub fn write_buffer<T: bytemuck::Pod>(&self,handle: Handle<Buffer>,data: T){
        let uniform = self.asset_manager.buffers.get(handle).unwrap();
        self.queue.write_buffer(&uniform.buffer, 0, bytemuck::cast_slice(&[data]));
    }

    pub fn create_compute_pipeline(&mut self,builder: ComputePipelineBuilder) -> Handle<ComputePipeline>{
        let pipeline = builder.build(&self.device,&self.asset_manager);
        self.asset_manager.compute_pipelines.insert(pipeline)
    }

    pub fn create_compute_bind_group(&mut self, builder: ComputeBindGroupBuilder) -> Handle<ComputeBindGroup>{
        let bind_group = builder.build(&self.device,&self.asset_manager);
        self.asset_manager.compute_bind_groups.insert(bind_group)
    }

}

