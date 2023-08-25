use sglc_shared::{ChunkMapping, Voxels};

pub fn clear(chunk_count: usize, chunk_mapping: &mut ChunkMapping, voxels: &mut Voxels) {
    for c in &mut chunk_mapping.0 {
        *c = 0;
    }

    for v in &mut voxels.0[0] {
        *v = 2;
    }

    if chunk_count > 0 {
        for c in &mut voxels.0[1..chunk_count] {
            for v in c {
                *v = 0;
            }
        }
    }
}
