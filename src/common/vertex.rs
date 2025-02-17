use crate::common::coords::*;
use bevy::{math::Vec3, prelude::Color};

use super::voxel_material::VoxelMaterial;

// #[derive(Eq, PartialEq, PartialOrd, Ord)]
#[derive(Clone)]
pub struct Vertex {
    pub pos: WorldCoords,
    pub normal: Vec3,
    pub color: Color,
    pub voxel_material: VoxelMaterial,
}
