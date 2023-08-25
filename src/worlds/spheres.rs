use crate::{World, Voxels, ChunkMapping};
use sglc_hotcode::clear::clear;
use sglc_hotcode::place_one_sphere::place_one_sphere;

use winit::event::VirtualKeyCode;
use winit::event::ElementState;


#[derive(Default)]
pub struct Spheres {
    has_generated: bool,
    chunk_count: usize,
}

impl World for Spheres {
    fn fill_in_voxels(&mut self, chunk_mapping: &mut ChunkMapping, voxels: &mut Voxels) {
        if self.has_generated { return };

        self.has_generated = true;

        clear(self.chunk_count, chunk_mapping, voxels); 

        let mut new_chunk = 1;

        for n in 0..200 {
            place_one_sphere(n % 10, &mut new_chunk, chunk_mapping, voxels);
        }

        self.chunk_count = new_chunk;
    }

    fn keyboard_input(&mut self, code: VirtualKeyCode, state: ElementState) {
        use winit::event::VirtualKeyCode::*;
        use winit::event::ElementState::*;

        match code {
            X if state == Pressed => {
                self.has_generated = false;
            },
            _ => {},
        }
    }
}


