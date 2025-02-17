use crate::common::voxel_material::VoxelMaterial;

#[derive(Clone, Copy)]
pub struct Voxel {
    pub value: f32,
    pub material: VoxelMaterial,
}

impl Default for Voxel {
    fn default() -> Self {
        Self {
            value: 0.0,
            material: VoxelMaterial::AIR,
        }
    }
}

/// Create a voxel given only its value (a f32)
impl From<f32> for Voxel {
    fn from(value: f32) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}
