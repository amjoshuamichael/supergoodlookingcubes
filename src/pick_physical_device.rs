use std::sync::Arc;

use vulkano::device::{DeviceExtensions, QueueFlags};
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::swapchain::Surface;
use vulkano::{instance::Instance, device::physical::PhysicalDevice};

pub struct DeviceInfo {
    pub device: Arc<PhysicalDevice>,
    pub graphics_queue_index: u32,
}

pub const REQUIRED_EXTENSIONS: DeviceExtensions = DeviceExtensions {
    khr_swapchain: true,
    ..DeviceExtensions::empty()
};

pub fn pick_best_physical_device(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>,
) -> DeviceInfo {
    instance
        .enumerate_physical_devices()
        .expect("could not enumerate devices")
        .filter(|p| p.supported_extensions().contains(&REQUIRED_EXTENSIONS))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                // Find the first first queue family that is suitable.
                // If none is found, `None` is returned to `filter_map`,
                // which disqualifies this physical device.
                .position(|(i, q)| {
                    q.queue_flags.contains(QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|q| DeviceInfo {
                    device: p.clone(),
                    graphics_queue_index: q as u32,
                })
        })
        .min_by_key(|DeviceInfo { device, .. }| match device.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,

            // `PhysicalDeviceType` is a non-exhaustive enum. Thus, one should
            // match wildcard `_` to catch all unknown device types.
            _ => 4,
        })
        .expect("no device available")
}
