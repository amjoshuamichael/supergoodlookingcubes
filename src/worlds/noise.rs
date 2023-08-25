use crate::{World, Voxels, ChunkMapping};

pub struct Noise {
    time: u32,
}

impl World for Noise {
    fn fill_in_voxels(&mut self, chunk_mapping: &mut ChunkMapping, voxels: &mut Voxels) {
        
    }
}
