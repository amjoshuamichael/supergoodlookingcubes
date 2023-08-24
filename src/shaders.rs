use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[derive(BufferContents, Vertex, Default, Copy, Clone, Debug)]
#[repr(C)]
pub struct MyVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: glam::Vec3,
}

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 460

            layout(location = 0) in vec3 position;
            layout(location = 0) out vec3 positionOut;

            layout(set = 1, binding = 0) uniform Pallete {
                vec3 color[256];
            } pallete;

            layout(set = 1, binding = 1) uniform CameraData {
                float aspect_ratio;
                float yaw;
                uint _padding;
                uint _padding2;
                vec3 position;
                mat4 camera;
                mat4 proj;
                mat4 camRot;
            } cam;

            void main() {
                vec4 cameraSpacePosition = cam.proj * cam.camera * vec4(position, 1.0);
                
                positionOut = position;
                gl_Position = vec4(cameraSpacePosition.xy, 0.0, cameraSpacePosition.z / 10.0);
            }
        ",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 460

            layout(location = 0) out vec4 f_color;
            layout(location = 0) in vec3 startPos;

            const uint CHUNK_SIZE_ONE = 8;
            const uint CHUNK_SIZE = CHUNK_SIZE_ONE * CHUNK_SIZE_ONE * CHUNK_SIZE_ONE;
            const uint CHUNK_COUNT_ONE = 4;
            const uint CHUNK_COUNT = CHUNK_COUNT_ONE * CHUNK_COUNT_ONE * CHUNK_COUNT_ONE;
            const uint WORLD_SIZE_ONE = CHUNK_SIZE_ONE * CHUNK_COUNT_ONE;
            const uint WORLD_SIZE = CHUNK_SIZE * CHUNK_COUNT;

            layout(set = 0, binding = 0) buffer ChunkMapping {
                uint data[1];
            } chunk_mapping;

            layout(set = 0, binding = 1) buffer Voxels {
                uint data[CHUNK_SIZE * CHUNK_COUNT];
            } voxels;

            layout(set = 1, binding = 0) uniform Pallete {
                vec3 color[256];
            } pallete;

            layout(set = 1, binding = 1) uniform CameraData {
                float aspect_ratio;
                float yaw;
                uint _padding;
                uint _padding2;
                vec3 position;
                mat4 camera;
                mat4 proj;
                mat4 camRot;
            } cam;

            uint voxel_unit_at(vec3 _pos) {
                if (_pos.x < 0.0 || _pos.x > WORLD_SIZE_ONE - 1
                    || _pos.y < 0.0 || _pos.y > WORLD_SIZE_ONE - 1
                    || _pos.z < 0.0 || _pos.z > WORLD_SIZE_ONE - 1) {
                    return 0;
                }

                uvec3 pos = uvec3(_pos);
                uvec3 chunkPos = pos / CHUNK_SIZE_ONE;
                uvec3 posInChunk = pos % CHUNK_SIZE;

                uint chunk_index = uint(
                    (chunkPos.x + chunkPos.y * CHUNK_SIZE_ONE + chunkPos.z * CHUNK_SIZE_ONE * CHUNK_SIZE_ONE) * CHUNK_SIZE
                );

                uint voxel_index = uint(
                    posInChunk.x + posInChunk.y * CHUNK_SIZE_ONE + posInChunk.z * CHUNK_SIZE_ONE * CHUNK_SIZE_ONE
                );

                return uint(voxels.data[chunk_mapping.data[chunk_index] * CHUNK_SIZE + voxel_index]);
            }

            float size_of_min_dimension(vec3 vector) {
                return min(vector.x, min(vector.y, vector.z));
            }
            
            void main() {
                float fov = 1.0;
                vec3 rd = normalize((
                    cam.proj * 
                    cam.camRot * 
                    vec4((startPos - cam.position), 1.0)
                ).xyz);
                vec3 ro = startPos;

                //if (pallete.color[69] != vec3(1.0)) {
                //    vec2 color = startPos.xy / 8.0;

                //    f_color = vec4(rd, 1.0);
                //    return;
                //}

                ro -= rd * 2;

                vec3 check_point = floor(ro);

                float xy = rd.x / rd.y;
                float yz = rd.y / rd.z;
                float zx = rd.z / rd.x;
                float xz = rd.x / rd.z;
                float yx = rd.y / rd.x;
                float zy = rd.z / rd.y;

                vec3 ray_unit_step_size = vec3(
                    sqrt(1 + zx * zx + yx * yx),
                    sqrt(1 + xy * xy + zy * zy),
                    sqrt(1 + xz * xz + yz * yz)
                );
                vec3 step = sign(rd);
                vec3 ray_length = (step * (check_point - ro) + (step / 2 + 0.5)) * ray_unit_step_size;

                vec3 comp;
                uint unit_at_check_point;
                for (int i = 0; i < WORLD_SIZE; i++) {
                    comp = vec3(bvec3(
                        ray_length.x < ray_length.y && ray_length.x <= ray_length.z,
                        ray_length.y < ray_length.z && ray_length.y <= ray_length.x,
                        ray_length.z < ray_length.x && ray_length.z <= ray_length.y
                    ));

                    check_point += comp * step;

                    unit_at_check_point = voxel_unit_at(check_point);
                    if(unit_at_check_point != 0) {
                        f_color = vec4(pallete.color[unit_at_check_point], 1.0);
                        return;
                    }

                    ray_length += comp * ray_unit_step_size;
                };

                f_color = vec4(0.0);
            }
        ",
    }
}
