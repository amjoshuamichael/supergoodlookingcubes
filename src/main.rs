#![feature(type_name_of_val)]

use std::sync::Arc;

pub trait Length {
    const LEN: usize;
}

impl<T, const LENGTH: usize> Length for [T; LENGTH] {
    const LEN: usize = LENGTH;
}

const CHUNK_SIZE_ONE: usize = 8;
const CHUNK_SIZE: usize = CHUNK_SIZE_ONE.pow(3);
const CHUNK_COUNT_ONE: usize = 4;
const CHUNK_COUNT: usize = CHUNK_COUNT_ONE.pow(3);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ChunkMapping([u32; CHUNK_COUNT]);
unsafe impl bytemuck::Zeroable for ChunkMapping {}
unsafe impl bytemuck::Pod for ChunkMapping {}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Voxels([[u32; CHUNK_SIZE]; CHUNK_COUNT]);
unsafe impl bytemuck::Zeroable for Voxels {}
unsafe impl bytemuck::Pod for Voxels {}

use camera_data::CameraData;
use command_buffer::get_command_buffers;
use pick_physical_device::{pick_best_physical_device, REQUIRED_EXTENSIONS};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::{VulkanLibrary, swapchain, sync};
use vulkano::buffer::{Buffer, BufferUsage, BufferCreateInfo};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::device::{QueueCreateInfo, DeviceCreateInfo, Device};
use vulkano::image::view::ImageView;
use vulkano::image::{ImageUsage, SwapchainImage, AttachmentImage};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::{StandardMemoryAllocator, MemoryUsage, AllocationCreateInfo};
use vulkano::pipeline::{GraphicsPipeline, Pipeline};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::render_pass::{RenderPass, Framebuffer, FramebufferCreateInfo, Subpass};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{SwapchainCreateInfo, Swapchain, CompositeAlphas, CompositeAlpha, AcquireError, SwapchainPresentInfo};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent, KeyboardInput, ElementState};
use winit::window::WindowBuilder;
use winit::event_loop::{ControlFlow, EventLoop};
use vulkano_win::VkSurfaceBuild;
use vulkano::sync::{GpuFuture, FlushError};
use glam::{Vec3, Quat, Mat4};

mod pick_physical_device;
mod shaders;
mod command_buffer;
mod camera_data;
mod create_vertex_buffer;

use shaders::MyVertex;
use create_vertex_buffer::set_vertex_buffer;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;
const ASPECT_RATIO: f32 = WIDTH as f32 / HEIGHT as f32;

fn main() {
    let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
    let required_extensions = vulkano_win::required_extensions(&library);
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..Default::default()
        },
    )
    .expect("failed to create instance");

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .with_title("poopoo haha")
        .with_inner_size(LogicalSize::new(WIDTH * 3, HEIGHT * 3))
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();

    let physical_device = pick_best_physical_device(&instance, &surface);

    let (device, mut queues) = Device::new(
        physical_device.device.clone(),
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index: physical_device.graphics_queue_index,
                ..Default::default()
            }],
            enabled_extensions: REQUIRED_EXTENSIONS,
            ..Default::default()
        },
    )
    .expect("failed to create device");

    let memory_allocator = StandardMemoryAllocator::new_default(device.clone());
    let cmd_buffer_allocator = StandardCommandBufferAllocator::new(
        device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default(),
    );

    let queue = queues.next().unwrap();

    let capabilities = physical_device.device
        .surface_capabilities(&surface, Default::default())
        .expect("failed to get surface capabilities");

    let dimensions = [WIDTH, HEIGHT];
    let composite_alpha = 
        pick_best_composite_alpha(capabilities.supported_composite_alpha).unwrap();
    let image_format = Some(
        physical_device.device
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0,
    );

    let depth_buffer = ImageView::new_default(
        AttachmentImage::transient(&memory_allocator, dimensions, vulkano::format::Format::D16_UNORM).unwrap(),
    ).unwrap();

    let (swapchain, images) = Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            // How many buffers to use in the swapchain
            min_image_count: capabilities.min_image_count + 1,
            image_format,
            image_extent: dimensions.into(),
            // What the images are going to be used for
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha,
            ..Default::default()
        },
    )
    .unwrap();

    let render_pass = get_render_pass(device.clone(), &swapchain);
    let framebuffers = get_framebuffers(&images, &render_pass, &depth_buffer);

    let vertex_buffer = Buffer::from_iter(
        &memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        //[
        //    MyVertex { position: Vec3::new(1.0, 1.0, 0.0) },
        //    MyVertex { position: Vec3::new(-1.0, 1.0, 0.0) },
        //    MyVertex { position: Vec3::new(1.0, -1.0, 0.0) },
        //    MyVertex { position: Vec3::new(-1.0, 1.0, 0.0) },
        //    MyVertex { position: Vec3::new(1.0, -1.0, 0.0) },
        //    MyVertex { position: Vec3::new(-1.0, -1.0, 0.0) },
        //],
        [MyVertex::default(); CHUNK_COUNT * 6],
    )
    .unwrap();

    let (chunk_mapping_buffer, voxels_buffer) = {
        let chunk_mapping_buffer = Buffer::new_unsized::<ChunkMapping>(
            &memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER | BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            std::mem::size_of::<ChunkMapping>() as u64,
        ).unwrap();

        let voxels_buffer = Buffer::new_unsized::<Voxels>(
            &memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER | BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            std::mem::size_of::<Voxels>() as u64,
        ).unwrap();

        let mut chunk_mapping = chunk_mapping_buffer.write().unwrap();
        
        chunk_mapping.0[0] = 2;
        chunk_mapping.0[1] = 2;
        chunk_mapping.0[5] = 2;
        //for c in 0..chunk_mapping.0.len() {
        //    chunk_mapping.0[c] = fastrand::u32(0..(8 as u32)); 

        //    if c % 8 == 0 {
        //        chunk_mapping.0[c] = 4; 
        //    } else {
        //        chunk_mapping.0[c] = 3; 
        //    }
        //}

        let mut voxels = voxels_buffer.write().unwrap();

        let chunk = &mut voxels.0[2];

        for v in 0..chunk.len() {
            if v % 2 == 0 {
                chunk[v] = fastrand::u32(1..6);
            }
        }

        for v in 0..chunk.len() {
            if v % 2 == 0 {
                chunk[v] = fastrand::u32(1..6);
            }
        }

        std::mem::drop(voxels);
        std::mem::drop(chunk_mapping);

        (chunk_mapping_buffer, voxels_buffer)
    };

    let pallete = {
        let mut pallete = Vec::<[f32; 4]>::new();
        pallete.push([0.0, 0.0, 0.0, 0.0]);
        pallete.push([0.07, 0.17, 0.20, 0.0]);
        pallete.push([0.13, 0.28, 0.43, 0.0]);
        pallete.push([0.16, 0.26, 0.58, 0.0]);
        pallete.push([0.30, 0.64, 0.85, 0.0]);
        pallete.push([0.27, 0.81, 0.79, 0.0]);

        if pallete.len() != 256 {
            pallete.push([0.0, 0.0, 0.0, 0.0]);
        }

        pallete
    };

    let camera_data = CameraData {
        aspect_ratio: HEIGHT as f32 / WIDTH as f32, 
        proj: Mat4::perspective_rh(0.7, std::f32::consts::FRAC_PI_2, 0.1, 1.0),
        position: Vec3::new(0.0, 0.0, -1.0),
        ..Default::default()
    };

    let pallete_buffer = Buffer::from_iter(
        &memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        pallete.into_iter(),
    ).unwrap();

    let camera_data_buffer = Buffer::from_iter(
        &memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        [camera_data].into_iter(),
    ).unwrap();

    let vs = shaders::vs::load(device.clone()).expect("failed to create shader module");
    let fs = shaders::fs::load(device.clone()).expect("failed to create shader module");

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [WIDTH as f32, HEIGHT as f32],
        depth_range: 0.0..1.0,
    };

    let pipeline = get_pipeline(
        device.clone(),
        vs.clone(),
        fs.clone(),
        render_pass.clone(),
        viewport.clone(),
    );

    let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
    let pipeline_layout = pipeline.layout();
    let descriptor_set_layouts = pipeline_layout.set_layouts();

    let blocks_descriptor_set = PersistentDescriptorSet::new(
        &descriptor_set_allocator,
        descriptor_set_layouts.get(0).unwrap().clone(),
        [
            WriteDescriptorSet::buffer(0, chunk_mapping_buffer.clone()),
            WriteDescriptorSet::buffer(1, voxels_buffer.clone()),
        ],
    ).unwrap();
    let render_descriptor_set = PersistentDescriptorSet::new(
        &descriptor_set_allocator,
        descriptor_set_layouts.get(1).unwrap().clone(),
        [
            WriteDescriptorSet::buffer(0, pallete_buffer.clone()),
            WriteDescriptorSet::buffer(1, camera_data_buffer.clone()),
        ],
    ).unwrap();

    let command_buffers = get_command_buffers(
        &queue,
        &pipeline,
        &framebuffers,
        &vertex_buffer,
        &blocks_descriptor_set,
        &render_descriptor_set,
        &cmd_buffer_allocator,
    );

    let mut recreate_swapchain = false;

    let mut time_avg = 0;
    let mut vertex_count = 0;
    let mut passed_frames = 0;
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                if recreate_swapchain {
                    println!("recreation is supposed to happen rn");
                    recreate_swapchain = false;
                }
                *control_flow = ControlFlow::Exit;
            },
            Event::WindowEvent { 
                event: WindowEvent::KeyboardInput { 
                    input: KeyboardInput {
                        virtual_keycode: Some(keycode),
                        state: ElementState::Pressed,
                        ..
                    }, 
                    .. 
                }, 
                .. 
            }  => {
                use winit::event::VirtualKeyCode::*;
                let camera_data = &mut camera_data_buffer.write().unwrap()[0];
                let camera_quat = Quat::from_rotation_y(-camera_data.yaw);

                match keycode {
                    A => {
                        camera_data.position -= camera_quat * Vec3::X;
                    },
                    D => {
                        camera_data.position += camera_quat * Vec3::X;
                    },
                    Q => {
                        camera_data.position += camera_quat * Vec3::Y;
                    },
                    E => {
                        camera_data.position -= camera_quat * Vec3::Y;
                    },
                    W => {
                        camera_data.position -= camera_quat * Vec3::Z;
                    },
                    S => {
                        camera_data.position += camera_quat * Vec3::Z;
                    },
                    R => {
                        camera_data.yaw -= 0.1;
                    },
                    F => {
                        camera_data.yaw += 0.1;
                    },
                    _ => {},
                }
            }
            Event::MainEventsCleared => {
                set_vertex_buffer(
                    &*chunk_mapping_buffer.read().unwrap(),
                    &mut *vertex_buffer.write().unwrap(),
                    &mut vertex_count,
                );

                {
                    let camera_data = &mut camera_data_buffer.write().unwrap()[0];
                    let camera_position = camera_data.position;
                    let camera_rotation = camera_data.quat();

                    let camera_matrix = Mat4::from_quat(camera_rotation) * Mat4::from_translation(-camera_position);
                    camera_data.camera = camera_matrix;
                    camera_data.proj = Mat4::perspective_lh(0.7, ASPECT_RATIO, 1.0, 100.0);
                    camera_data.rot = Mat4::from_quat(camera_data.neg_quat());
                }

                let execution_time = std::time::Instant::now();

                let (image_i, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("failed to acquire next image: {e}"),
                    };

                if suboptimal {
                    recreate_swapchain = true;
                }

                let execution = sync::now(device.clone())
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffers[image_i as usize].clone())
                    .unwrap()
                    .then_swapchain_present(
                        queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_i),
                    )
                    .then_signal_fence_and_flush();

                match execution {
                    Ok(future) => {
                        future.wait(None).unwrap();  // wait for the GPU to finish
                        let time = execution_time.elapsed().as_micros();
                        time_avg += time;
                        passed_frames += 1;

                        if time_avg >= 1000000 {
                            let time_per_frame_in_millis = time_avg / passed_frames;
                            println!("{}fps", 1000000 / time_per_frame_in_millis);
                            time_avg = 0;
                            passed_frames = 0;
                        }
                    }
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                    }
                    Err(e) => {
                        println!("Failed to flush future: {e}");
                    }
                }
            }
            _ => ()
        }
    });
}

fn get_render_pass(device: Arc<Device>, swapchain: &Arc<Swapchain>) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device,
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.image_format(), // set the format the same as the swapchain
                samples: 1,
            },
             depth: {
                load: Clear,
                store: DontCare,
                format: vulkano::format::Format::D16_UNORM,
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth},
        },
    )
    .unwrap()
}

fn get_framebuffers(
    images: &[Arc<SwapchainImage>],
    render_pass: &Arc<RenderPass>,
    depth_buffer: &Arc<ImageView<AttachmentImage>>,
) -> Vec<Arc<Framebuffer>> {
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view, depth_buffer.clone()],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}

fn get_pipeline(
    device: Arc<Device>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
    render_pass: Arc<RenderPass>,
    viewport: Viewport,
) -> Arc<GraphicsPipeline> {
    GraphicsPipeline::start()
        .vertex_input_state(MyVertex::per_vertex())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .depth_stencil_state(DepthStencilState::simple_depth_test())
        .render_pass(Subpass::from(render_pass, 0).unwrap())
        .build(device)
        .unwrap()
}

pub fn pick_best_composite_alpha(from: CompositeAlphas) -> Option<CompositeAlpha> {
    if from.intersects(CompositeAlphas::OPAQUE) {
        Some(CompositeAlpha::Opaque)
    } else if from.intersects(CompositeAlphas::INHERIT) {
        Some(CompositeAlpha::Inherit)
    } else if from.intersects(CompositeAlphas::PRE_MULTIPLIED) {
        Some(CompositeAlpha::PreMultiplied)
    } else if from.intersects(CompositeAlphas::POST_MULTIPLIED) {
        Some(CompositeAlpha::PostMultiplied)
    } else {
        None
    }
}
