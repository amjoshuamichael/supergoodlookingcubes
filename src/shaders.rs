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
                float pitch;
                uint _padding2;
                vec3 position;
                mat4 camera;
                mat4 proj;
                mat4 camRot;
            } cam;

            void main() {
                vec4 cameraSpacePosition = cam.proj * cam.camera * vec4(position, 1.0);
                
                positionOut = position;
                gl_Position = cameraSpacePosition;
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
            const uint CHUNK_COUNT_ONE = 128;
            const uint CHUNK_COUNT = CHUNK_COUNT_ONE * CHUNK_COUNT_ONE * CHUNK_COUNT_ONE;
            const uint WORLD_SIZE_ONE = CHUNK_SIZE_ONE * CHUNK_COUNT_ONE;
            const uint WORLD_SIZE = CHUNK_SIZE * CHUNK_COUNT;

            layout(set = 0, binding = 0) buffer ChunkMapping {
                uint data[CHUNK_COUNT];
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
                float pitch;
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
                    return 1;
                }

                uvec3 pos = uvec3(_pos);
                uvec3 chunkPos = pos / CHUNK_SIZE_ONE;
                uvec3 posInChunk = pos % CHUNK_SIZE_ONE;

                uint chunk_index = uint(
                    chunkPos.x + chunkPos.y * CHUNK_COUNT_ONE + chunkPos.z * CHUNK_COUNT_ONE * CHUNK_COUNT_ONE
                );

                uint voxel_index = uint(
                    posInChunk.x + posInChunk.y * CHUNK_SIZE_ONE + posInChunk.z * CHUNK_SIZE_ONE * CHUNK_SIZE_ONE
                );

                return 
                    uint(voxels.data[chunk_mapping.data[chunk_index] * CHUNK_SIZE + voxel_index]);
            }

            float size_of_min_dimension(vec3 vector) {
                return min(vector.x, min(vector.y, vector.z));
            }

            struct hit {
                vec3 pos;
                vec3 normal;
                bool air;

                uint unit_code;
            };

            hit hit_in_direction(vec3 ro, vec3 rd) {
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
                for (int i = 0; i < WORLD_SIZE_ONE * 3; i++) {
                    comp = vec3(bvec3(
                        ray_length.x < ray_length.y && ray_length.x <= ray_length.z,
                        ray_length.y < ray_length.z && ray_length.y <= ray_length.x,
                        ray_length.z < ray_length.x && ray_length.z <= ray_length.y
                    ));

                    check_point += comp * step;

                    unit_at_check_point = voxel_unit_at(check_point);
                    if(unit_at_check_point >= 1) {
                        if (unit_at_check_point == 2) {
                            // we are in a chunk filled with air
                            uvec3 first_chunk = uvec3(check_point) / CHUNK_SIZE_ONE;

                            ray_length += comp * ray_unit_step_size;

                            uvec3 current_chunk = first_chunk;
                            uint moves = 0;
                            while (current_chunk == first_chunk && moves < 8) { 
                                comp = vec3(bvec3(
                                    ray_length.x < ray_length.y && ray_length.x <= ray_length.z,
                                    ray_length.y < ray_length.z && ray_length.y <= ray_length.x,
                                    ray_length.z < ray_length.x && ray_length.z <= ray_length.y
                                ));
                                
                                check_point += comp * step;
                                ray_length += comp * ray_unit_step_size;

                                current_chunk = uvec3(check_point) / CHUNK_SIZE_ONE;
                                moves++;
                            }

                            continue;
                        }

                        return hit(ro + rd * size_of_min_dimension(ray_length), - comp * step, unit_at_check_point == 1, unit_at_check_point);
                    }

                    ray_length += comp * ray_unit_step_size;
                };

                return hit(vec3(0.0), vec3(0.0), true, 0);
            }
            
            void main() {
                float fov = 1.0;
                vec2 screenpos = (gl_FragCoord.xy - vec2(160.0, 90.0)) / vec2(160.0);
                vec3 rd = (
                    cam.camRot * 
                    vec4(
                        normalize(
                            vec3(
                                screenpos.xy,
                                1.54
                            )
                        ), 
                        1000.0
                    )
                ).xyz;

                vec3 ro = cam.position;
                ro += rd * (distance(cam.position, startPos) - 2);

                hit albedo = hit_in_direction(ro, rd);

                if (albedo.air) {
                    f_color = vec4(vec3(0.1), 1.0);
                    return;
                }

                if (albedo.unit_code > 2) {
                    hit reflection = hit_in_direction(albedo.pos + albedo.normal, -normalize(vec3(1.0)));
                    
                    f_color = vec4(pallete.color[albedo.unit_code].xyz, 1.0);

                    if (!reflection.air) {
                        f_color /= 2.0; 
                    }

                    return;
                }
            }
        ",
    }
}
