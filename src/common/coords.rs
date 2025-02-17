use crate::common::constants::*;
use ordered_float::OrderedFloat;

use bevy::prelude::*;
use std::ops::{Add, Mul, Sub};

use crate::common::coords_neighbours_iter::CoordsNeighboursIter;

// Generic 3d coordinate type
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug, Default)]
pub struct Coords<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

/// Generic 2d coordinate type, mostly used on the xz plane (disregarding the y axis)
#[derive(Clone, Copy, Debug, Default)]
pub struct Coords2D<T> {
    pub x: T,
    pub z: T,
}

// Real-World coordinates in f32 (OrderedFloat allows for coordinates comparison)
pub type WorldCoords = Coords<OrderedFloat<f32>>;

// Real-World coordinates expressed in i64
pub type IWorldCoords = Coords<i64>;

// Coordinates used for Chunk position in the world
pub type ChunkCoords = Coords<i64>;

// Coordinates used for voxel position inside a chunk
// Value should range from 0 to CHUNK_VSIZE (33)
pub type VoxelCoords = Coords<u8>;

// Signed Coords with no predefined context
pub type ICoords = Coords<i32>;

/// Coordinates used for voxel 2d position on the xz plane inside a chunk
pub type VoxelCoords2D = Coords2D<u8>;

impl<T> Coords<T>
where
    T: Sub<Output = T> + Mul<Output = T> + Copy,
{
    #[inline]
    pub fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - rhs.y * self.z,
            y: self.z * rhs.x - rhs.z * self.x,
            z: self.x * rhs.y - rhs.x * self.y,
        }
    }
}

impl WorldCoords {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            z: OrderedFloat(z),
        }
    }

    /* returns the vanilla floats version of itself */
    pub fn into_inners(self) -> Coords<f32> {
        Coords {
            x: self.x.into_inner(),
            y: self.y.into_inner(),
            z: self.z.into_inner(),
        }
    }

    /* returns the vanilla floats version of itself in an array format */
    pub fn into_inners_arr(self) -> [f32; 3] {
        [
            self.x.into_inner(),
            self.y.into_inner(),
            self.z.into_inner(),
        ]
    }
}

impl VoxelCoords {
    pub fn new(x: u8, y: u8, z: u8) -> Self {
        Self { x, y, z }
    }
}

impl ChunkCoords {
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }
}

impl<T: Add<Output = T>> Add for Coords<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub<VoxelCoords> for VoxelCoords {
    type Output = ICoords; // result can be negative
    #[inline]
    fn sub(self, rhs: Self) -> ICoords {
        ICoords {
            x: (self.x as i32).sub(rhs.x as i32),
            y: (self.y as i32).sub(rhs.y as i32),
            z: (self.z as i32).sub(rhs.z as i32),
        }
    }
}

impl Sub<WorldCoords> for WorldCoords {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x.sub(rhs.x),
            y: self.y.sub(rhs.y),
            z: self.z.sub(rhs.z),
        }
    }
}

impl Mul<f32> for WorldCoords {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self {
            x: OrderedFloat(self.x.into_inner().mul(rhs)),
            y: OrderedFloat(self.y.into_inner().mul(rhs)),
            z: OrderedFloat(self.z.into_inner().mul(rhs)),
        }
    }
}

impl WorldCoords {
    /// Returns the real world coordinates given chunk coords and voxel coords inside the
    /// coresponding chunk
    pub fn from_voxel(chunk_coords: ChunkCoords, voxel_coords: VoxelCoords) -> Self {
        Self {
            x: OrderedFloat(CHUNK_SIZE as f32 * chunk_coords.x as f32 + voxel_coords.x as f32),
            y: OrderedFloat(CHUNK_SIZE as f32 * chunk_coords.y as f32 + voxel_coords.y as f32),
            z: OrderedFloat(CHUNK_SIZE as f32 * chunk_coords.z as f32 + voxel_coords.z as f32),
        }
    }
}

impl From<Vec3> for WorldCoords {
    fn from(vec: Vec3) -> Self {
        Self {
            x: OrderedFloat(vec.x),
            y: OrderedFloat(vec.y),
            z: OrderedFloat(vec.z),
        }
    }
}

impl From<WorldCoords> for VoxelCoords {
    fn from(world_coords: WorldCoords) -> Self {
        Self {
            x: (world_coords.x.into_inner() as i32).rem_euclid(CHUNK_SIZE as i32) as u8,
            y: (world_coords.y.into_inner() as i32).rem_euclid(CHUNK_SIZE as i32) as u8,
            z: (world_coords.z.into_inner() as i32).rem_euclid(CHUNK_SIZE as i32) as u8,
        }
    }
}

impl From<WorldCoords> for VoxelCoords2D {
    fn from(world_coords: WorldCoords) -> Self {
        Self {
            // rem_euclid is a modulo operation, taking negative numbers into account
            z: (world_coords.z.into_inner().floor() as i32).rem_euclid(CHUNK_SIZE as i32) as u8,
            x: (world_coords.x.into_inner().floor() as i32).rem_euclid(CHUNK_SIZE as i32) as u8,
        }
    }
}

/// TODO: Maybe handle error for usize::from, see what is the idiomatic way for error handling in
/// Rust

/// Delinearize an index into 3d voxel coordinates in a chunk
/// The given index must be between 0 and CHUNK_VVOLUME
impl From<usize> for VoxelCoords {
    fn from(index: usize) -> Self {
        Self {
            x: (index / (CHUNK_VAREA)) as u8,
            y: ((index % (CHUNK_VAREA)) / CHUNK_VSIZE) as u8,
            z: (index % CHUNK_VSIZE) as u8,
        }
    }
}

/// Delinearize an index into 2d voxel coordinates in a chunk
/// The given index must be between 0 and CHUNK_VAREA
impl From<usize> for VoxelCoords2D {
    fn from(index: usize) -> Self {
        Self {
            x: (index % (CHUNK_VSIZE)) as u8,
            z: (index / CHUNK_VSIZE) as u8,
        }
    }
}

/// Linearize VoxelCoords into the corresponding index,
/// expected container size must be of size CHUNK_VVOLUME
impl From<VoxelCoords> for usize {
    fn from(voxel_coords: VoxelCoords) -> Self {
        return voxel_coords.z as usize
            + voxel_coords.y as usize * CHUNK_VSIZE
            + voxel_coords.x as usize * CHUNK_VSIZE * CHUNK_VSIZE;
    }
}

/// Linearize VoxelCoords2D into the corresponding index,
/// expected container size must be of size CHUNK_VAREA
impl From<VoxelCoords2D> for usize {
    fn from(voxel_coords_2d: VoxelCoords2D) -> Self {
        voxel_coords_2d.z as usize + voxel_coords_2d.x as usize * CHUNK_VSIZE
    }
}

/// Convert VoxelCoords into its corresponding 2d version, ignoring the y component
impl From<VoxelCoords> for VoxelCoords2D {
    fn from(voxel_coords: VoxelCoords) -> Self {
        Self {
            x: voxel_coords.x,
            z: voxel_coords.z,
        }
    }
}

impl ChunkCoords {
    // Returns the chunk position for a given World coordinates (vec3)
    pub fn from_vec(v: Vec3) -> ChunkCoords {
        let x: i64 = if v.x > 0.0 {
            v.x as i64 / CHUNK_SIZE as i64
        } else {
            v.x as i64 / CHUNK_SIZE as i64 - 1
        };
        let y = if v.y > 0.0 {
            v.y as i64 / CHUNK_SIZE as i64
        } else {
            v.y as i64 / CHUNK_SIZE as i64 - 1
        };
        let z = if v.z > 0.0 {
            v.z as i64 / CHUNK_SIZE as i64
        } else {
            v.z as i64 / CHUNK_SIZE as i64 - 1
        };

        Self { x, y, z }
    }

    pub fn iter_neighbours(&self, include_self: bool) -> CoordsNeighboursIter {
        CoordsNeighboursIter::new(self.clone(), include_self)
    }
}
