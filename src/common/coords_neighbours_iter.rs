use crate::common::coords::*;

/*
 * Iterate over all 6 neighbours of specified position (north, south, east, west, up, down)
 * possibly including starting position
 */
#[derive(Clone, Copy)]
pub struct CoordsNeighboursIter {
    x: i64,
    y: i64,
    z: i64,
    pos: ChunkCoords,
    include_self: bool,
}

impl CoordsNeighboursIter {
    pub fn new(pos: ChunkCoords, include_self: bool) -> Self {
        Self {
            x: -1,
            y: -1,
            z: -1,
            pos,
            include_self,
        }
    }
}

impl Iterator for CoordsNeighboursIter {
    type Item = ChunkCoords;

    fn next(&mut self) -> Option<ChunkCoords> {
        if self.z > 1 {
            return None;
        }
        let result = ChunkCoords::new(
            self.pos.x + self.x,
            self.pos.y + self.y,
            self.pos.z + self.z,
        );

        self.x += 1;
        if self.x > 1 {
            self.x = -1;
            self.y += 1;
            if self.y > 1 {
                self.y = -1;
                self.z += 1;
            }
        } else if !self.include_self && self.x == 0 && self.y == 0 && self.z == 0 {
            self.x += 1;
        }

        return Some(result);
    }
}
