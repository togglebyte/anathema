use super::GraphicsCtx;
use crate::wgpu::model::INDICES;
use crate::wgpu::texture::Textures;

pub struct Renderer;

impl Renderer {
    pub(crate) fn render(&mut self, ctx: &mut GraphicsCtx) -> Result<(), ()> {
        #[cfg(feature = "profiling")]
        puffin::profile_function!();

        let clear_color = [0.2, 0.3, 0.4, 1.0];

        // Update view_matrix uniform
        // NOTE: do we need this?
        ctx.queue.write_buffer(
            &mut ctx.projection_buffer,
            0,
            bytemuck::cast_slice(&[ctx.camera.to_matrix()]),
        );

        // // NOTE: do we need this?
        // ctx.queue.write_buffer(
        //     &mut ctx.sprites.instance_buffer,
        //     0,
        //     bytemuck::cast_slice(&ctx.sprites.sprite_cache),
        // );

        let output = ctx.surface.get_current_texture().unwrap(); // TODO: add `?` back in when
                                                                 // result is decided upon;

        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command encoder"),
        });

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: clear_color[0],
                        g: clear_color[1],
                        b: clear_color[2],
                        a: clear_color[3],
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None, // TODO add depth stencil
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // -----------------------------------------------------------------------------
        //   - Render pipeline -
        //   * One pipeline per material
        //   * Group textures by material
        // -----------------------------------------------------------------------------
        for material in ctx.materials.iter() {
            for (sprite, texture) in ctx.sprites(&material.sprites) {
                render_pass.set_pipeline(&material.pipeline);
                render_pass.set_bind_group(0, &texture.bind_group, &[]);
                render_pass.set_bind_group(1, &ctx.camera_bind_group, &[]);

                panic!("if there is only one sprite then don't use an instance buffer");

                render_pass.set_vertex_buffer(0, ctx.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, ctx.sprites.instance_buffer.slice(..));
                render_pass.set_index_buffer(ctx.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..ctx.sprites.len() as u32);
            }
        }

        drop(render_pass);
        ctx.queue.submit(Some(encoder.finish()));

        output.present();
        Ok(())
    }
}
