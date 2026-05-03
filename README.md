# easy-gpu
A simple wgpu wrapper to help manage rendering assets and remove boilerplate code from projects.

## Usage

example of a simple renderer with a camera and two quads
```rust
let mut renderer = pollster::block_on(Renderer::new(window.clone()));

let vertices = [
    Vertex::new([-1.0, -1.0]),
    Vertex::new([1.0, -1.0]),
    Vertex::new([1.0, 1.0]),
    Vertex::new([-1.0, 1.0])
];

let indices = [0, 1, 2, 0, 2, 3];

let mesh = renderer.create_mesh(&vertices, &indices);

let camera = Camera::new();

let camera_buffer = renderer.create_buffer_with_contents(
    BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    bytemuck::cast_slice(&[camera])
);

let shader = renderer.load_shader(include_str!("shader.wgsl"));

let pipeline_builder = RenderPipelineBuilder::new(shader.clone())
    .vertex_layout(Vertex::buffer_layout())
    .vertex_layout(Instance::buffer_layout())
    .material_layout(&[uniform(0)])

let pipeline = renderer.create_render_pipeline(pipeline_builder);

let material_builder = MaterialBuilder::new(pipeline)
    .uniform(0,camera_buffer);

let material = renderer.create_material(material_builder);

let instances = vec![
  Instance::new(0.,0.)
  Instance::new(1.,0.)
];

let instance_buffer = renderer.create_buffer_with_contents(
    BufferUsages::VERTEX | BufferUsages::STORAGE,
    bytemuck::cast_slice(instances.as_slice())
);
```
example usage of the above renderer

```rust
let frame = renderer.begin_frame();

frame.draw(
    instance_buffer,
    material,
    mesh,
    0..2
);

renderer.render();
