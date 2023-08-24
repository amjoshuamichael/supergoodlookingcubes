use std::sync::Arc;

use vulkano::buffer::Subbuffer;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, SubpassContents,
    RenderPassBeginInfo,
};

use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Queue;
use vulkano::pipeline::{GraphicsPipeline, Pipeline};
use vulkano::render_pass::Framebuffer;

use crate::shaders::MyVertex;

pub fn get_command_buffers(
    queue: &Arc<Queue>,
    pipeline: &Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    vertex_buffer: &Subbuffer<[MyVertex]>,
    descriptor_set_1: &Arc<PersistentDescriptorSet>,
    descriptor_set_2: &Arc<PersistentDescriptorSet>,
    allocator: &StandardCommandBufferAllocator,
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    framebuffers
        .iter()
        .map(|framebuffer| {
            let mut builder = AutoCommandBufferBuilder::primary(
                allocator,
                queue.queue_family_index(),
                CommandBufferUsage::MultipleSubmit,
            )
            .unwrap();

            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.1, 0.1, 0.1, 1.0].into()), Some(1.0.into())],
                        ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                    },
                    SubpassContents::Inline,
                )
                .unwrap()
                .bind_pipeline_graphics(pipeline.clone())
                .bind_descriptor_sets(
                    vulkano::pipeline::PipelineBindPoint::Graphics,
                    pipeline.layout().clone(),
                    0,
                    (descriptor_set_1.clone(), descriptor_set_2.clone()),
                )
                .bind_vertex_buffers(0, vertex_buffer.clone())
                .draw(vertex_buffer.len() as u32, 1, 0, 0)
                .unwrap()
                .end_render_pass()
                .unwrap();

            Arc::new(builder.build().unwrap())
        })
        .collect()
}
