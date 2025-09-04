use anyhow::{anyhow, Result};
use log::*;
use std::collections::HashSet;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::{PhysicalDevice, SurfaceKHR};

use crate::infrastructure::constants::DEVICE_EXTENSIONS;
use crate::infrastructure::error::SuitabilityError;
use crate::infrastructure::queue_family_indices::QueueFamilyIndices;
use crate::infrastructure::swapchain::SwapchainSupport;

//================================================
// Physical Device
//================================================

/// Picks the physical device
///
/// # Safety
/// Check the vulkan docs for safety info
pub unsafe fn pick_physical_device(
    instance: &Instance,
    surface: SurfaceKHR,
) -> Result<PhysicalDevice> {
    for physical_device in instance.enumerate_physical_devices()? {
        let properties = instance.get_physical_device_properties(physical_device);

        if let Err(error) = check_physical_device(instance, surface, physical_device) {
            warn!(
                "Skipping physical device (`{}`): {}",
                properties.device_name, error
            );
        } else {
            info!("Selected physical device (`{}`).", properties.device_name);
            return Ok(physical_device);
        }
    }

    Err(anyhow!("Failed to find suitable physical device."))
}

unsafe fn check_physical_device(
    instance: &Instance,
    surface: SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    QueueFamilyIndices::get(instance, surface, physical_device)?;
    check_physical_device_extensions(instance, physical_device)?;

    let support = SwapchainSupport::get(instance, surface, physical_device)?;
    if support.formats.is_empty() || support.present_modes.is_empty() {
        return Err(anyhow!(SuitabilityError("Insufficient swapchain support.")));
    }

    let features = instance.get_physical_device_features(physical_device);
    if features.sampler_anisotropy != vk::TRUE {
        return Err(anyhow!(SuitabilityError("No sampler anisotropy.")));
    }

    Ok(())
}

unsafe fn check_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    let extensions = instance
        .enumerate_device_extension_properties(physical_device, None)?
        .iter()
        .map(|e| e.extension_name)
        .collect::<HashSet<_>>();
    if DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e)) {
        Ok(())
    } else {
        Err(anyhow!(SuitabilityError(
            "Missing required device extensions."
        )))
    }
}

/// Gets the max msaa samples supported for the device
///
/// # Safety
/// Check the vulkan docs for safety info
pub unsafe fn get_max_msaa_samples(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> vk::SampleCountFlags {
    let properties = instance.get_physical_device_properties(physical_device);
    let counts = properties.limits.framebuffer_color_sample_counts
        & properties.limits.framebuffer_depth_sample_counts;

    [
        vk::SampleCountFlags::_64,
        vk::SampleCountFlags::_32,
        vk::SampleCountFlags::_16,
        vk::SampleCountFlags::_8,
        vk::SampleCountFlags::_4,
        vk::SampleCountFlags::_2,
    ]
    .iter()
    .cloned()
    .find(|c| counts.contains(*c))
    .unwrap_or(vk::SampleCountFlags::_1)
}
