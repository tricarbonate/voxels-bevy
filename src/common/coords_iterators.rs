use crate::common::coords::*;
use crate::common::coords_neighbours::HasNeighbours;

/*
 * Iterate through all neihgbouring coordinates inside a sphere of
 * specified radius
 */
pub struct AroundCoordsIterator {
    start: ChunkCoords,
    current: ChunkCoords,
    current_radius: i64,
    done: bool,
    radius: i64,
}

impl AroundCoordsIterator {
    pub fn new(start: ChunkCoords, radius: usize) -> Self {
        Self {
            radius: radius as i64,
            start,
            done: false,
            current_radius: 0,
            current: ChunkCoords::new(0, -(radius as i64), 0),
        }
    }
    pub fn is_done(&self) -> bool {
        self.done
    }
}

impl Iterator for AroundCoordsIterator {
    type Item = ChunkCoords;

    fn next(&mut self) -> Option<ChunkCoords> {
        let r = self.current_radius;
        if self.radius == r {
            self.done = true;
            return None;
        }

        let y_r = self.radius - r + 1;

        let new_pos = match self.current {
            p if p.y < y_r => p.get_up(),
            mut p if p.z == r && p.x == -r => {
                p.y = -y_r + 1;
                println!("current radius changes: {}", self.current_radius);
                self.current_radius += 1;
                p.get_north()
            }

            mut p if p.x < r && p.z == r => {
                p.y = -y_r;
                p.get_east()
            }

            mut p if p.z > -r && p.x == r => {
                p.y = -y_r;
                p.get_south()
            }

            mut p if p.x > -r && p.z == -r => {
                p.y = -y_r;
                p.get_west()
            }

            mut p if p.z < r && p.x == -r => {
                p.y = -y_r;
                p.get_north()
            }

            _ => {
                panic!("unreachable");
            }
        };

        self.current = new_pos;

        Some(new_pos + self.start)
    }
}

impl ChunkCoords {
    pub fn iter_around(&self, radius: usize) -> AroundCoordsIterator {
        AroundCoordsIterator::new(*self, radius)
    }
}

impl VoxelCoords {
    pub fn iter_around(&self, radius: usize) -> AroundCoordsIterator {
        AroundCoordsIterator::new(
            ChunkCoords::new(self.x as i64, self.y as i64, self.z as i64),
            radius,
        )
    }
}
