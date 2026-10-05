//! The 64-byte per-instance record shared with the shaders.

/// pos (loc 0), size (1), color (2), uv (3), params (4); params.x is the quad kind.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Instance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub uv: [f32; 4],
    pub params: [f32; 4],
}

pub const KIND_SOLID: f32 = 0.0;
pub const KIND_TEXTURED: f32 = 1.0;
pub const KIND_BALL: f32 = 2.0;
pub const KIND_SHADOW: f32 = 3.0;
pub const KIND_ROUNDED: f32 = 4.0;

const _: () = assert!(std::mem::size_of::<Instance>() == 64);
