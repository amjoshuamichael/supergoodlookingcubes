use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[derive(BufferContents, Vertex, Default, Copy, Clone, Debug)]
#[repr(C)]
pub struct MyVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: glam::Vec3,
}

pub const CHUNK_SIZE_ONE: usize = 8;
pub const CHUNK_SIZE: usize = CHUNK_SIZE_ONE.pow(3);
pub const CHUNK_COUNT_ONE: usize = 128;
pub const CHUNK_COUNT: usize = CHUNK_COUNT_ONE.pow(3);
pub const WORLD_SIZE_ONE: usize = CHUNK_SIZE_ONE * CHUNK_COUNT_ONE;
pub const WORLD_SIZE: usize = CHUNK_SIZE * CHUNK_COUNT;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ChunkMapping(pub [u32; CHUNK_COUNT]);
unsafe impl bytemuck::Zeroable for ChunkMapping {}
unsafe impl bytemuck::Pod for ChunkMapping {}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Voxels(pub [[u32; CHUNK_SIZE]; CHUNK_COUNT]);
unsafe impl bytemuck::Zeroable for Voxels {}
unsafe impl bytemuck::Pod for Voxels {}

pub const VERTEX_COUNT: usize = CHUNK_COUNT * 72 / 8;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vertices(pub [MyVertex; VERTEX_COUNT]);
unsafe impl bytemuck::Zeroable for Vertices {}
unsafe impl bytemuck::Pod for Vertices {}


