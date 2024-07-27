use bytemuck::{Pod, Zeroable};
use wgpu::{Buffer, Device};
use wgpu::util::DeviceExt;


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
   pub position: [f32; 3],
}
impl Vertex {
   pub fn desc() -> wgpu::VertexBufferLayout<'static> {
      wgpu::VertexBufferLayout {
         array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
         step_mode: wgpu::VertexStepMode::Vertex,
         attributes: &[
            wgpu::VertexAttribute {
               offset: 0,
               shader_location: 0,
               format: wgpu::VertexFormat::Float32x3,
            },
         ]
      }
   }
}


pub struct VertexPackage {
   pub vertex_buffer: Buffer,
   pub index_buffer: Buffer,
   pub num_indices: u32,
   pub num_vertices: u32,
}
impl VertexPackage {
   pub fn new(device: &Device, vertices: &[Vertex], indices: &[u16]) -> Self {
      let vertex_buffer = device.create_buffer_init(
         &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
         }
      );
      let num_vertices = vertices.len() as u32;

      let index_buffer = device.create_buffer_init(
         &wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
         }
      );
      let num_indices = indices.len() as u32;


      Self {
         vertex_buffer,
         num_indices,
         num_vertices,
         index_buffer,
      }
   }
}