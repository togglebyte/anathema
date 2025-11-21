use anathema_geometry::{Pos, Region, Size};
use unicode_width::UnicodeWidthChar;

use super::GraphicsCtx;
use crate::wgpu::buffer::{Buffer, Diff};
use crate::wgpu::font::Font;
use crate::wgpu::model::{INDEX_COUNT, INDICES};
use crate::wgpu::texture::Textures;
use crate::wgpu::{MaterialId, Sprite, State, Style};

pub struct Renderer {
    pub(crate) size: Size,
    pub(crate) font: Font,
    pub(crate) front: Buffer,
    pub(crate) back: Buffer,
}

impl Renderer {
    pub(crate) fn new(size: Size, font: Font) -> Self {
        Self {
            size,
            font,
            front: Buffer::new(size.width as usize, size.height as usize),
            back: Buffer::new(size.width as usize, size.height as usize),
        }
    }

    pub(crate) fn resize(&mut self, size: Size) {
        self.size = size;
    }

    pub(crate) fn render(&mut self, ctx: &mut GraphicsCtx) -> Result<(), ()> {
        self.present(ctx)?;

        self.front.clear_dirty_rows();
        self.back.clear_dirty_rows();

        Ok(())
    }

    fn render_partial(&mut self) {
        let mut buffer = vec![];
        self.front.sync_buffers(&self.back, &mut buffer);

        let mut last_y = 0;

        for (y, diff) in buffer {
            match diff {
                Diff::ClearRow => {
                    // write_style(Style::reset(), &mut self.output);
                    // self.output.queue(cursor::MoveTo(0, y as u16)).unwrap();
                    // _ = self.output.queue(Print(&self.empty_line));
                }
                Diff::ClearRange(range) => {
                    // write_style(Style::reset(), &mut self.output);
                    // self.output.queue(cursor::MoveTo(range.start as u16, y as u16)).unwrap();
                    // _ = self.output.queue(Print(&self.empty_line[range]));
                }
                Diff::Write(range) => {
                    for x in range {
                        let index = y * self.size.width as usize + x;
                        let cell = &self.front[index];

                        // write_style(cell.style, &mut self.output);

                        // write the character
                        // match &cell.state {
                        //     super::State::Empty | super::State::Continuation => (),
                        //     super::State::Char(c) => _ = self.output.queue(Print(c)),
                        //     super::State::Cluster(cluster) => _ = self.output.queue(Print(cluster)),
                        // }
                    }
                }
            }
        }
    }

    pub(crate) fn present(&mut self, ctx: &mut GraphicsCtx) -> Result<(), ()> {
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

        // Camera
        render_pass.set_bind_group(1, &ctx.camera_bind_group, &[]);

        // Vertices
        render_pass.set_vertex_buffer(0, ctx.vertex_buffer.slice(..));
        render_pass.set_index_buffer(ctx.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        // -----------------------------------------------------------------------------
        //   - Render pipeline -
        //   * One pipeline per material
        //   * Group textures by material
        // -----------------------------------------------------------------------------

        for (material, texture, sprite_buffer, sprite_count) in ctx.something() {
            render_pass.set_pipeline(&material.pipeline);
            render_pass.set_bind_group(0, &texture.bind_group, &[]);

            render_pass.set_vertex_buffer(1, sprite_buffer.slice(..));
            render_pass.draw_indexed(0..INDEX_COUNT, 0, 0..sprite_count);
        }

        // for material in ctx.materials.iter() {
        //     for (sprite, texture) in ctx.sprites(&material.sprites) {
        //         render_pass.set_pipeline(&material.pipeline);
        //         render_pass.set_bind_group(0, &texture.bind_group, &[]);

        //         // render_pass.set_vertex_buffer(0, ctx.vertex_buffer.slice(..));
        //         render_pass.set_vertex_buffer(1, ctx.sprites.instance_buffer.slice(..));
        //         // render_pass.set_index_buffer(ctx.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        //         render_pass.draw_indexed(0..INDEX_COUNT, 0, 0..ctx.sprites.len() as u32);
        //     }
        // }

        drop(render_pass);
        ctx.queue.submit(Some(encoder.finish()));

        output.present();

        Ok(())
    }

    fn style_region(&mut self, region: Region, style: Style) {
        let from_y = region.from.y as usize;
        let to_y = region.to.y as usize;
        let width = (region.to.x - region.from.x) as usize;
        let from_x = region.from.x as usize;
        let to_x = region.to.x as usize;
        for y in from_y..to_y {
            let mut insert = self.back.begin_insert(y);
            insert.write_style(from_x..to_x, style);
        }
    }
}
