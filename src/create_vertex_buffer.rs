use glam::Vec3;

use crate::{shaders::MyVertex, ChunkMapping, CHUNK_COUNT_ONE, CHUNK_SIZE_ONE};

pub fn set_vertex_buffer(
    chunk_mapping: &ChunkMapping,
    vertex_buffer: &mut [MyVertex],
    vertex_count: &mut usize,
) {
    pub fn vert(xyz: Vec3, add: (f32, f32, f32)) -> MyVertex {
        MyVertex {
            position: xyz + Vec3::from(add) * CHUNK_SIZE_ONE as f32,
        }
    }

    let mut vertex_index = 0;

    for c in 0..chunk_mapping.0.len() {
        if chunk_mapping.0[c] == 0 {
            break;
        }

        let z = c / (CHUNK_COUNT_ONE * CHUNK_COUNT_ONE);
        let y = (c / CHUNK_COUNT_ONE) % CHUNK_COUNT_ONE;
        let x = c % CHUNK_COUNT_ONE;
        let xyz = Vec3::new(x as f32, y as f32, z as f32);

        vertex_buffer[vertex_index + 00] = vert(xyz, (1.0, 1.0, 0.0));
        vertex_buffer[vertex_index + 01] = vert(xyz, (0.0, 1.0, 0.0));
        vertex_buffer[vertex_index + 02] = vert(xyz, (1.0, 0.0, 0.0));
        vertex_buffer[vertex_index + 03] = vert(xyz, (0.0, 1.0, 0.0));
        vertex_buffer[vertex_index + 04] = vert(xyz, (1.0, 0.0, 0.0));
        vertex_buffer[vertex_index + 05] = vert(xyz, (0.0, 0.0, 0.0));

        vertex_buffer[vertex_index + 06] = vert(xyz, (1.0, 0.0, 1.0));
        vertex_buffer[vertex_index + 07] = vert(xyz, (1.0, 0.0, 0.0));
        vertex_buffer[vertex_index + 08] = vert(xyz, (0.0, 0.0, 1.0));
        vertex_buffer[vertex_index + 09] = vert(xyz, (1.0, 0.0, 0.0));
        vertex_buffer[vertex_index + 10] = vert(xyz, (0.0, 0.0, 1.0));
        vertex_buffer[vertex_index + 11] = vert(xyz, (0.0, 0.0, 0.0));

        vertex_buffer[vertex_index + 12] = vert(xyz, (0.0, 1.0, 1.0));
        vertex_buffer[vertex_index + 13] = vert(xyz, (0.0, 1.0, 0.0));
        vertex_buffer[vertex_index + 14] = vert(xyz, (0.0, 0.0, 1.0));
        vertex_buffer[vertex_index + 15] = vert(xyz, (0.0, 1.0, 0.0));
        vertex_buffer[vertex_index + 16] = vert(xyz, (0.0, 0.0, 1.0));
        vertex_buffer[vertex_index + 17] = vert(xyz, (0.0, 0.0, 0.0));


        vertex_index += 6;
    }

    if *vertex_count > vertex_index {
        for v in &mut vertex_buffer[vertex_index..*vertex_count] {
            *v = MyVertex::default();
        }
    }

    *vertex_count = vertex_index;
}


// MyVertex { position: [1.0, 1.0] },
// MyVertex { position: [-1.0, 1.0] },
// MyVertex { position: [1.0, -1.0] },
// MyVertex { position: [-1.0, 1.0] },
// MyVertex { position: [1.0, -1.0] },
// MyVertex { position: [-1.0, -1.0] },

