use anyhow::Result;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk;
use vulkanalia::Device;

//================================================
// Framebuffers
//================================================

pub unsafe fn create_framebuffers(
    device: &Device,
    render_pass: vk::RenderPass,
    image_views: &Vec<vk::ImageView>,
    height: u32,
    width: u32,
) -> Result<Vec<vk::Framebuffer>> {
    Ok(image_views
        .iter()
        .map(|i| {
            let attachments = &[*i];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(attachments)
                .width(width)
                .height(height)
                .layers(1);

            device.create_framebuffer(&create_info, None)
        })
        .collect::<Result<Vec<_>, _>>()?)
}
