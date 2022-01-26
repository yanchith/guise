use alloc::vec::Vec;

use crate::convert::cast_u32;
use crate::core::math::Rect;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(bytemuck::Zeroable, bytemuck::Pod)]
pub struct Command {
    pub scissor_rect: Rect,
    pub texture_id: u64,
    pub index_count: u32,
    // NB: Explicit padding, so that bytemuck doesn't cast uninitialized padding
    // bytes to something.
    pub _pad: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coord: [f32; 2],
    pub color: u32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DrawList {
    commands: Vec<Command>,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl DrawList {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn draw_rect(
        &mut self,
        rect: Rect,
        texture_rect: Rect,
        color: u32,
        scissor_rect: Rect,
        texture_id: u64,
    ) {
        let tl_position = [rect.x, rect.y];
        let tl_tex_coord = [texture_rect.x, texture_rect.y];

        let tr_position = [rect.max_x(), rect.y];
        let tr_tex_coord = [texture_rect.max_x(), texture_rect.y];

        let bl_position = [rect.x, rect.max_y()];
        let bl_tex_coord = [texture_rect.x, texture_rect.max_y()];

        let br_position = [rect.max_x(), rect.max_y()];
        let br_tex_coord = [texture_rect.max_x(), texture_rect.max_y()];

        let index_base = cast_u32(self.vertices.len());

        self.vertices.push(Vertex {
            position: bl_position,
            tex_coord: bl_tex_coord,
            color,
        });
        self.vertices.push(Vertex {
            position: br_position,
            tex_coord: br_tex_coord,
            color,
        });
        self.vertices.push(Vertex {
            position: tr_position,
            tex_coord: tr_tex_coord,
            color,
        });
        self.vertices.push(Vertex {
            position: tl_position,
            tex_coord: tl_tex_coord,
            color,
        });

        // 0, 1, 2
        let i1 = index_base;
        let i2 = index_base + 1;
        let i3 = index_base + 2;
        // 2, 3, 0
        let i4 = index_base + 2;
        let i5 = index_base + 3;
        let i6 = index_base;

        self.indices.push(i1);
        self.indices.push(i2);
        self.indices.push(i3);
        self.indices.push(i4);
        self.indices.push(i5);
        self.indices.push(i6);

        if let Some(ref mut last_command) = self.commands.last_mut() {
            if last_command.scissor_rect == scissor_rect && last_command.texture_id == texture_id {
                last_command.index_count += 6;
            } else {
                self.commands.push(Command {
                    scissor_rect,
                    texture_id,
                    index_count: 6,
                    _pad: 0,
                });
            }
        } else {
            self.commands.push(Command {
                scissor_rect,
                texture_id,
                index_count: 6,
                _pad: 0,
            });
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
        self.vertices.clear();
        self.indices.clear();
    }
}
