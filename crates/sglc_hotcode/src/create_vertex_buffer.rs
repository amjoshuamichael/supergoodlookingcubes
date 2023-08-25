use glam::Vec3;

use sglc_shared::{ChunkMapping, CHUNK_COUNT_ONE, CHUNK_SIZE_ONE, Vertices};
use sglc_shared::MyVertex;
use crate::pos_to_index::index_to_pos;

pub fn set_vertex_buffer(
    chunk_mapping: &ChunkMapping,
    vertex_buffer: &mut Vertices,
    vertex_count: &mut usize,
) {
    pub fn vert(xyz: Vec3, add: (f32, f32, f32)) -> MyVertex {
        MyVertex {
            position: (xyz + Vec3::from(add)) * CHUNK_SIZE_ONE as f32,
        }
    }

    let mut vertex_index = 0;

    for c in 0..chunk_mapping.0.len() {
        if chunk_mapping.0[c] == 0 {
            continue;
        }

        let xyz = index_to_pos(c, CHUNK_COUNT_ONE);

        vertex_buffer.0[vertex_index + 00] = vert(xyz, (1.0, 1.0, 0.0));
        vertex_buffer.0[vertex_index + 01] = vert(xyz, (0.0, 1.0, 0.0));
        vertex_buffer.0[vertex_index + 02] = vert(xyz, (1.0, 0.0, 0.0));
        vertex_buffer.0[vertex_index + 03] = vert(xyz, (0.0, 1.0, 0.0));
        vertex_buffer.0[vertex_index + 04] = vert(xyz, (1.0, 0.0, 0.0));
        vertex_buffer.0[vertex_index + 05] = vert(xyz, (0.0, 0.0, 0.0));

        vertex_buffer.0[vertex_index + 06] = vert(xyz, (1.0, 1.0, 1.0));
        vertex_buffer.0[vertex_index + 07] = vert(xyz, (0.0, 1.0, 1.0));
        vertex_buffer.0[vertex_index + 08] = vert(xyz, (1.0, 0.0, 1.0));
        vertex_buffer.0[vertex_index + 09] = vert(xyz, (0.0, 1.0, 1.0));
        vertex_buffer.0[vertex_index + 10] = vert(xyz, (1.0, 0.0, 1.0));
        vertex_buffer.0[vertex_index + 11] = vert(xyz, (0.0, 0.0, 1.0));

        vertex_buffer.0[vertex_index + 12] = vert(xyz, (0.0, 1.0, 1.0));
        vertex_buffer.0[vertex_index + 13] = vert(xyz, (0.0, 1.0, 0.0));
        vertex_buffer.0[vertex_index + 14] = vert(xyz, (0.0, 0.0, 1.0));
        vertex_buffer.0[vertex_index + 15] = vert(xyz, (0.0, 1.0, 0.0));
        vertex_buffer.0[vertex_index + 16] = vert(xyz, (0.0, 0.0, 1.0));
        vertex_buffer.0[vertex_index + 17] = vert(xyz, (0.0, 0.0, 0.0));

        vertex_buffer.0[vertex_index + 18] = vert(xyz, (1.0, 1.0, 1.0));
        vertex_buffer.0[vertex_index + 19] = vert(xyz, (1.0, 1.0, 0.0));
        vertex_buffer.0[vertex_index + 20] = vert(xyz, (1.0, 0.0, 1.0));
        vertex_buffer.0[vertex_index + 21] = vert(xyz, (1.0, 1.0, 0.0));
        vertex_buffer.0[vertex_index + 22] = vert(xyz, (1.0, 0.0, 1.0));
        vertex_buffer.0[vertex_index + 23] = vert(xyz, (1.0, 0.0, 0.0));

        vertex_buffer.0[vertex_index + 24] = vert(xyz, (1.0, 0.0, 1.0));
        vertex_buffer.0[vertex_index + 25] = vert(xyz, (1.0, 0.0, 0.0));
        vertex_buffer.0[vertex_index + 26] = vert(xyz, (0.0, 0.0, 1.0));
        vertex_buffer.0[vertex_index + 27] = vert(xyz, (1.0, 0.0, 0.0));
        vertex_buffer.0[vertex_index + 28] = vert(xyz, (0.0, 0.0, 1.0));
        vertex_buffer.0[vertex_index + 29] = vert(xyz, (0.0, 0.0, 0.0));

        vertex_buffer.0[vertex_index + 30] = vert(xyz, (1.0, 1.0, 1.0));
        vertex_buffer.0[vertex_index + 31] = vert(xyz, (1.0, 1.0, 0.0));
        vertex_buffer.0[vertex_index + 32] = vert(xyz, (0.0, 1.0, 1.0));
        vertex_buffer.0[vertex_index + 33] = vert(xyz, (1.0, 1.0, 0.0));
        vertex_buffer.0[vertex_index + 34] = vert(xyz, (0.0, 1.0, 1.0));
        vertex_buffer.0[vertex_index + 35] = vert(xyz, (0.0, 1.0, 0.0));

        vertex_index += 36;
    }

    if *vertex_count > vertex_index {
        for v in &mut vertex_buffer.0[vertex_index..*vertex_count] {
            *v = MyVertex::default();
        }
    }

    *vertex_count = vertex_index;
}
