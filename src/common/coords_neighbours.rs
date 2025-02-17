use crate::common::coords::*;

/*
 * Trait for non-float coordinates
 */
pub trait HasNeighbours {
    fn get_north(&self) -> Self;
    fn get_south(&self) -> Self;
    fn get_east(&self) -> Self;
    fn get_west(&self) -> Self;
    fn get_up(&self) -> Self;
    fn get_down(&self) -> Self;
}

impl HasNeighbours for ChunkCoords {
    fn get_north(&self) -> Self {
        ChunkCoords::new(self.x, self.y, self.z + 1)
    }
    fn get_south(&self) -> Self {
        ChunkCoords::new(self.x, self.y, self.z - 1)
    }
    fn get_east(&self) -> Self {
        ChunkCoords::new(self.x + 1, self.y, self.z)
    }
    fn get_west(&self) -> Self {
        ChunkCoords::new(self.x - 1, self.y, self.z)
    }
    fn get_up(&self) -> Self {
        ChunkCoords::new(self.x, self.y + 1, self.z)
    }
    fn get_down(&self) -> Self {
        ChunkCoords::new(self.x, self.y - 1, self.z)
    }
}

impl HasNeighbours for VoxelCoords {
    fn get_north(&self) -> Self {
        VoxelCoords::new(self.x, self.y, self.z + 1)
    }
    fn get_south(&self) -> Self {
        VoxelCoords::new(self.x, self.y, self.z - 1)
    }
    fn get_east(&self) -> Self {
        VoxelCoords::new(self.x + 1, self.y, self.z)
    }
    fn get_west(&self) -> Self {
        VoxelCoords::new(self.x - 1, self.y, self.z)
    }
    fn get_up(&self) -> Self {
        VoxelCoords::new(self.x, self.y + 1, self.z)
    }
    fn get_down(&self) -> Self {
        VoxelCoords::new(self.x, self.y - 1, self.z)
    }
}
