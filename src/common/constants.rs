pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;

// number of voxels needed along one axis
pub const CHUNK_VSIZE: usize = CHUNK_SIZE + 1;

// number of voxels needed along a plane of a chunk
pub const CHUNK_VAREA: usize = CHUNK_VSIZE * CHUNK_VSIZE;

// number of voxels needed for a whole chunk
pub const CHUNK_VVOLUME: usize = CHUNK_VSIZE * CHUNK_VSIZE * CHUNK_VSIZE;

pub const CHUNK_LOAD_AT_ONCE: usize = 6;
pub const CHUNK_LOAD_RANGE: usize = 6;

// Ground level, the ground can actually go below this height
pub const GROUND_LEVEL: f32 = -20.0;
pub const MAX_TERRAIN_HEIGHT: f32 = 20.0;
