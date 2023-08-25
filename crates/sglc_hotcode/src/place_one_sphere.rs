use sglc_shared::{WORLD_SIZE_ONE, CHUNK_COUNT_ONE, CHUNK_SIZE_ONE, ChunkMapping, Voxels};
use crate::pos_to_index::pos_to_index;
use glam::IVec3;

pub fn place_one_sphere(n: usize, new_chunk: &mut usize, chunk_mapping: &mut ChunkMapping, voxels: &mut Voxels) {
    let radius = fastrand::i32(20..60);
    let radius_squared = radius * radius;
    let center = IVec3::new(
        fastrand::i32((0 + radius)..(WORLD_SIZE_ONE as i32 - radius)),
        fastrand::i32((0 + radius)..(WORLD_SIZE_ONE as i32 - radius)),
        fastrand::i32((0 + radius)..(WORLD_SIZE_ONE as i32 - radius)),
    );

    let corner1 = center - IVec3::ONE * radius;
    let corner2 = center + IVec3::ONE * radius;
    let chunk1 = corner1 / 8;
    let chunk2 = corner2 / 8;

    for cx in chunk1.x..=chunk2.x {
        for cy in chunk1.y..=chunk2.y {
            for cz in chunk1.z..=chunk2.z {
                let chunk_index = pos_to_index((cx, cy, cz), CHUNK_COUNT_ONE);
                let chunk = IVec3::new(cx, cy, cz) * CHUNK_SIZE_ONE as i32;

                let mut placed_one = false;

                let center_of_chunk = chunk + IVec3::ONE * CHUNK_SIZE_ONE as i32 / 2;

                if ((center_of_chunk - center).length_squared()) < (radius - CHUNK_SIZE_ONE as i32).pow(2) {
                    continue;
                }

                for vx in 0..(CHUNK_SIZE_ONE as i32) {
                    for vy in 0..(CHUNK_SIZE_ONE as i32) {
                        for vz in 0..(CHUNK_SIZE_ONE as i32) {
                            let voxel = IVec3::new(vx, vy, vz);
                            let voxel_in_chunk = voxel + chunk;
                            let dist_squared = (voxel_in_chunk - center).length_squared();

                            if dist_squared < radius_squared {
                                voxels.0[*new_chunk][pos_to_index(voxel, CHUNK_SIZE_ONE)] = 
                                    ((dist_squared & 10) >> 1) as u32 + 3 + n as u32 * 2;
                                placed_one = true;
                            }
                        }
                    }
                }

                if placed_one {
                    chunk_mapping.0[chunk_index] = *new_chunk as u32;
                    *new_chunk += 1;
                }
            }
        }
    }
}
