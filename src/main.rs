use std::sync::Arc;

use command_buffer::get_command_buffers;
use pick_physical_device::{pick_best_physical_device, REQUIRED_EXTENSIONS};
use vulkano::{VulkanLibrary, swapchain, sync};
use vulkano::buffer::{Buffer, BufferUsage, BufferCreateInfo};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::device::{QueueCreateInfo, DeviceCreateInfo, Device};
use vulkano::image::view::ImageView;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::{StandardMemoryAllocator, MemoryUsage, AllocationCreateInfo};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::render_pass::{RenderPass, Framebuffer, FramebufferCreateInfo, Subpass};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{SwapchainCreateInfo, Swapchain, CompositeAlphas, CompositeAlpha, AcquireError, SwapchainPresentInfo};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::window::WindowBuilder;
use winit::event_loop::{ControlFlow, EventLoop};
use vulkano_win::VkSurfaceBuild;
use vulkano::sync::{GpuFuture, FlushError};

mod pick_physical_device;
mod shaders;
mod command_buffer;

use shaders::MyVertex;

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 1000;

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
        .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
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
    let framebuffers = get_framebuffers(&images, &render_pass);

    let vertices = {
        let mut vertices = Vec::new();

        const MAX: usize = 1000;

        for n in 0..MAX {
            vertices.push(MyVertex { position: [-0.5 + (n as f32 / MAX as f32), -0.5]});
            vertices.push(MyVertex { position: [0.0, 0.5]});
            vertices.push(MyVertex { position: [0.5, -0.25]});
        }

        println!("VERTICES COUNT: {}", MAX * 3);

        vertices
    };
    
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
        vertices,
    )
    .unwrap();

    let vs = shaders::vs::load(device.clone()).expect("failed to create shader module");
    let fs = shaders::fs::load(device.clone()).expect("failed to create shader module");

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [1000.0, 1000.0],
        depth_range: 0.0..1.0,
    };

    let pipeline = get_pipeline(
        device.clone(),
        vs.clone(),
        fs.clone(),
        render_pass.clone(),
        viewport.clone(),
    );

    let command_buffers = get_command_buffers(
        &queue,
        &pipeline,
        &framebuffers,
        &vertex_buffer,
        &cmd_buffer_allocator,
    );

    let mut recreate_swapchain = false;

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
            Event::MainEventsCleared => {
                vertex_buffer.write().unwrap()[0].position[0] -= 0.01;

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
        },
        pass: {
            color: [color],
            depth_stencil: {},
        },
    )
    .unwrap()
}

fn get_framebuffers(
    images: &[Arc<SwapchainImage>],
    render_pass: &Arc<RenderPass>,
) -> Vec<Arc<Framebuffer>> {
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
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
