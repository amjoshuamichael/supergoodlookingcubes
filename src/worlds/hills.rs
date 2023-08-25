use std::f32::consts::PI;

use crate::{World, CHUNK_COUNT_ONE, ChunkMapping, Voxels, CHUNK_SIZE_ONE};

#[derive(Copy, Clone, Default)]
pub struct Hills;

impl World for Hills {
    fn fill_in_voxels(&mut self, chunk_mapping: &mut ChunkMapping, voxels: &mut Voxels) {
        for x in 0..CHUNK_COUNT_ONE {
            for y in 0..CHUNK_COUNT_ONE {
                chunk_mapping.0[x + y * CHUNK_COUNT_ONE * CHUNK_COUNT_ONE + CHUNK_COUNT_ONE] = 
                    5;
            }
        }

        for x in 0..CHUNK_COUNT_ONE {
            for y in 0..CHUNK_COUNT_ONE {
                chunk_mapping.0[x + y * CHUNK_COUNT_ONE * CHUNK_COUNT_ONE] = 
                    ((x % 2) + (y % 2) * 2 + 1) as u32;
            }
        }

        fastrand::seed(12);

        for x in 0..CHUNK_SIZE_ONE {
            for y in 0..CHUNK_SIZE_ONE {
                for z in 0..CHUNK_SIZE_ONE {
                    voxels.0[5][x + y * CHUNK_SIZE_ONE + z * CHUNK_SIZE_ONE * CHUNK_SIZE_ONE] = fastrand::u32(1..5);
                }
            }
        }

        for (n, (o_x, o_z)) in [(0, 0), (1, 0), (0, 1), (1, 1)].into_iter().enumerate() {
            for x in 0..CHUNK_SIZE_ONE {
                for y in 0..CHUNK_SIZE_ONE {
                    for z in 0..CHUNK_SIZE_ONE {
                        let sin_x = x + o_x * CHUNK_SIZE_ONE;
                        let sin_z = z + o_z * CHUNK_SIZE_ONE;

                        if y as f32 <= 
                            (1.0 - (
                                ((sin_x as f32 * PI / 2. / 4.).cos()) *
                                ((sin_z as f32 * PI / 2. / 4.).cos())
                            )) * 4.
                            {
                            continue;
                        }

                        voxels.0[n + 1][x + y * CHUNK_SIZE_ONE + z * CHUNK_SIZE_ONE * CHUNK_SIZE_ONE] = fastrand::u32(1..5) + n as u32 * 4;
                    }
                }
            }
        }
    }
}
