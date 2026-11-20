// SPDX-License-Identifier: Apache-2.0
// Adapted from Havunen/gpui-ce, commit 28045891bfe87768a2c8e0aefd43403939844318,
// crates/gpui_apple/src/metal_renderer.rs (copy_texture, draw_clear_quad, pipelines).
// Copyright 2022 - 2025 Zed Industries, Inc.; Havunen and gpui-ce contributors.
// RustUI modifications: objc2-metal pipelines and pixel-exact, unblended passes.
use anyhow::{Context as _, Result};
use gpui::{DevicePixels, Size};
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_foundation::NSString;
use objc2_metal::{
    MTLCommandBuffer, MTLCommandEncoder, MTLDevice, MTLLibrary, MTLPixelFormat, MTLPrimitiveType,
    MTLRenderCommandEncoder, MTLRenderPipelineDescriptor, MTLRenderPipelineState, MTLScissorRect,
    MTLTexture,
};

pub(crate) struct Pipelines {
    copy: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    clear: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
}
impl Pipelines {
    pub(crate) fn new(
        device: &ProtocolObject<dyn MTLDevice>,
        library: &ProtocolObject<dyn MTLLibrary>,
    ) -> Result<Self> {
        let build = |fragment: &str| -> Result<_> {
            let vertex = library
                .newFunctionWithName(&NSString::from_str("damage_fullscreen_vertex"))
                .context("locating damage fullscreen vertex")?;
            let fragment_fn = library
                .newFunctionWithName(&NSString::from_str(fragment))
                .context("locating damage fragment")?;
            let descriptor = MTLRenderPipelineDescriptor::new();
            descriptor.setLabel(Some(&NSString::from_str(fragment)));
            descriptor.setVertexFunction(Some(&vertex));
            descriptor.setFragmentFunction(Some(&fragment_fn));
            // SAFETY: Every pipeline descriptor has color attachment zero.
            let attachment = unsafe { descriptor.colorAttachments().objectAtIndexedSubscript(0) };
            attachment.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
            attachment.setBlendingEnabled(false);
            device
                .newRenderPipelineStateWithDescriptor_error(&descriptor)
                .context("creating damage pipeline")
        };
        Ok(Self {
            copy: build("damage_copy_fragment")?,
            clear: build("damage_clear_fragment")?,
        })
    }
    pub(crate) fn copy(
        &self,
        buffer: &ProtocolObject<dyn MTLCommandBuffer>,
        source: &ProtocolObject<dyn MTLTexture>,
        target: &ProtocolObject<dyn MTLTexture>,
        size: Size<DevicePixels>,
    ) -> Result<()> {
        anyhow::ensure!(
            source.width() == target.width() && source.height() == target.height(),
            "damage copy dimensions differ"
        );
        let encoder =
            crate::metal_renderer::new_command_encoder_for_texture(buffer, target, size, None)?;
        encoder.setRenderPipelineState(&self.copy);
        unsafe {
            encoder.setFragmentTexture_atIndex(Some(source), 0);
            encoder.drawPrimitives_vertexStart_vertexCount(MTLPrimitiveType::Triangle, 0, 3);
        }
        encoder.endEncoding();
        Ok(())
    }
    pub(crate) fn clear(
        &self,
        buffer: &ProtocolObject<dyn MTLCommandBuffer>,
        target: &ProtocolObject<dyn MTLTexture>,
        size: Size<DevicePixels>,
        rect: MTLScissorRect,
        color: [f32; 4],
    ) -> Result<()> {
        anyhow::ensure!(
            rect.width > 0
                && rect.height > 0
                && rect
                    .x
                    .checked_add(rect.width)
                    .is_some_and(|x| x <= target.width())
                && rect
                    .y
                    .checked_add(rect.height)
                    .is_some_and(|y| y <= target.height()),
            "invalid damage clear scissor"
        );
        let encoder =
            crate::metal_renderer::new_command_encoder_for_texture(buffer, target, size, None)?;
        encoder.setRenderPipelineState(&self.clear);
        encoder.setScissorRect(rect);
        // SAFETY: Metal copies all sixteen bytes of this live float4; three vertices cover the viewport.
        unsafe {
            encoder.setFragmentBytes_length_atIndex(
                std::ptr::NonNull::from(&color).cast(),
                std::mem::size_of_val(&color),
                0,
            );
            encoder.drawPrimitives_vertexStart_vertexCount(MTLPrimitiveType::Triangle, 0, 3);
        }
        encoder.endEncoding();
        Ok(())
    }
}

#[cfg(any(test, feature = "test-support"))]
mod proof {
    use super::*;
    use crate::metal_renderer::MetalRenderer;
    use objc2_metal::{
        MTLBlitCommandEncoder, MTLBuffer, MTLCommandBufferStatus, MTLCommandQueue, MTLGPUFamily,
        MTLOrigin, MTLRegion, MTLResourceOptions, MTLSize, MTLStorageMode, MTLTextureDescriptor,
        MTLTextureUsage,
    };

    impl MetalRenderer {
        /// Headless Stage G copy/clear proof using the real damage pipelines.
        pub fn test_damage_shader_pixels() -> Result<()> {
            objc2::rc::autoreleasepool(|_| {
                let renderer = Self::new_headless(Default::default())?;
                let (width, height) = (64usize, 24usize);
                let size = gpui::size(DevicePixels(width as i32), DevicePixels(height as i32));
                let descriptor = MTLTextureDescriptor::new();
                unsafe {
                    descriptor.setWidth(width);
                    descriptor.setHeight(height);
                }
                descriptor.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
                descriptor.setStorageMode(
                    if cfg!(target_os = "ios")
                        || renderer.device.supportsFamily(MTLGPUFamily::Apple1)
                    {
                        MTLStorageMode::Shared
                    } else {
                        MTLStorageMode::Managed
                    },
                );
                descriptor.setUsage(MTLTextureUsage::ShaderRead | MTLTextureUsage::RenderTarget);
                let source = renderer
                    .device
                    .newTextureWithDescriptor(&descriptor)
                    .context("source texture")?;
                let target = renderer
                    .device
                    .newTextureWithDescriptor(&descriptor)
                    .context("target texture")?;
                let pixels = (0..width * height * 4)
                    .map(|i| ((i * 37 + i / 11) % 256) as u8)
                    .collect::<Vec<_>>();
                let region = MTLRegion {
                    origin: MTLOrigin { x: 0, y: 0, z: 0 },
                    size: MTLSize {
                        width,
                        height,
                        depth: 1,
                    },
                };
                unsafe {
                    source.replaceRegion_mipmapLevel_withBytes_bytesPerRow(
                        region,
                        0,
                        std::ptr::NonNull::from(pixels.as_slice()).cast(),
                        width * 4,
                    );
                }
                let command = renderer
                    .command_queue
                    .commandBuffer()
                    .context("copy command")?;
                renderer
                    .damage_pipelines
                    .copy(&command, &source, &target, size)?;
                let actual = read(&renderer, &command, &target)?;
                assert_eq!(
                    actual, pixels,
                    "copy pipeline must preserve every BGRA byte, including alpha"
                );
                let command = renderer
                    .command_queue
                    .commandBuffer()
                    .context("clear command")?;
                let rect = MTLScissorRect {
                    x: 3,
                    y: 5,
                    width: 11,
                    height: 7,
                };
                renderer
                    .damage_pipelines
                    .clear(&command, &target, size, rect, [1., 0., 1., 0.])?;
                let actual = read(&renderer, &command, &target)?;
                for y in 0..height {
                    for x in 0..width {
                        let i = (y * width + x) * 4;
                        let expected = if (3..14).contains(&x) && (5..12).contains(&y) {
                            &[255, 0, 255, 0][..]
                        } else {
                            &pixels[i..i + 4]
                        };
                        assert_eq!(&actual[i..i + 4], expected, "scissored clear at ({x},{y})");
                    }
                }
                Ok(())
            })
        }
    }
    fn read(
        renderer: &MetalRenderer,
        command: &ProtocolObject<dyn MTLCommandBuffer>,
        texture: &ProtocolObject<dyn MTLTexture>,
    ) -> Result<Vec<u8>> {
        let row = texture.width() * 4;
        let output = renderer
            .device
            .newBufferWithLength_options(
                row * texture.height(),
                MTLResourceOptions::StorageModeShared,
            )
            .context("readback buffer")?;
        let blit = command.blitCommandEncoder().context("readback encoder")?;
        unsafe {
            blit.copyFromTexture_sourceSlice_sourceLevel_sourceOrigin_sourceSize_toBuffer_destinationOffset_destinationBytesPerRow_destinationBytesPerImage(texture,0,0,MTLOrigin{x:0,y:0,z:0},MTLSize{width:texture.width(),height:texture.height(),depth:1},&output,0,row,row*texture.height());
        }
        blit.endEncoding();
        command.commit();
        command.waitUntilCompleted();
        anyhow::ensure!(
            command.status() == MTLCommandBufferStatus::Completed,
            "damage shader command failed: {:?}",
            command.error()
        );
        Ok(unsafe {
            std::slice::from_raw_parts(
                output.contents().as_ptr().cast::<u8>(),
                row * texture.height(),
            )
            .to_vec()
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn copy_and_scissored_clear_preserve_exact_pixels() -> anyhow::Result<()> {
        crate::metal_renderer::MetalRenderer::test_damage_shader_pixels()
    }
}
