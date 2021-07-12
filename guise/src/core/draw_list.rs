use alloc::vec::Vec;

use crate::core::math::Rect;

// TODO(yan): @Speed @Memory Think about generating primitive buffers instead of
// vertex buffers (as an optional alternative). This will make the backends more
// complicated, but will need to transfer much less data to the GPU.
//
// https://ourmachinery.com/post/ui-rendering-using-primitive-buffers/

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Draw {
        texture_id: u64,
        vertex_count: u32,
    },
    PushScissorRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    PopScissorRect,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable, bytemuck::Pod))]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coord: [f32; 2],
    pub color: u32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DrawList {
    commands: Vec<Command>,
    vertices: Vec<Vertex>,
    // TODO(yan): @Speed @Memory Have an index buffer.
}

impl DrawList {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            vertices: Vec::new(),
        }
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    pub fn draw_rect(&mut self, rect: Rect, tex_coord_rect: Rect, color: u32, texture_id: u64) {
        let vertices_start = self.vertices.len();
        let vertices_end = vertices_start + 6;

        let tl_position = [rect.x(), rect.y()];
        let tl_tex_coord = [tex_coord_rect.x(), tex_coord_rect.y()];

        let tr_position = [rect.max_x(), rect.y()];
        let tr_tex_coord = [tex_coord_rect.max_x(), tex_coord_rect.y()];

        let bl_position = [rect.x(), rect.max_y()];
        let bl_tex_coord = [tex_coord_rect.x(), tex_coord_rect.max_y()];

        let br_position = [rect.max_x(), rect.max_y()];
        let br_tex_coord = [tex_coord_rect.max_x(), tex_coord_rect.max_y()];

        // 0, 1, 2
        self.vertices.push(Vertex {
            position: tl_position,
            tex_coord: tl_tex_coord,
            color,
        });
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

        // 2, 3, 0
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

        self.commands.push(Command::Draw {
            vertex_count: 6,
            texture_id,
        });

        debug_assert_eq!(vertices_end, self.vertices.len());
    }

    pub fn push_scissor_rect(&mut self, rect: Rect) {
        self.commands.push(Command::PushScissorRect {
            x: rect.x(),
            y: rect.y(),
            width: rect.width(),
            height: rect.height(),
        })
    }

    pub fn pop_scissor_rect(&mut self) {
        // If we didn't draw anything inside the scissor rect, we don't need it.
        if let Some(Command::PushScissorRect { .. }) = self.commands.last() {
            self.commands.pop();
        } else {
            self.commands.push(Command::PopScissorRect);
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
        self.vertices.clear();
    }
}
