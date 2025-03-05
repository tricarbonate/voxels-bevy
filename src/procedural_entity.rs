use bevy::{ecs::component::Component, math::Vec3, utils::Instant};
use std::arch::x86_64::*;
use std::cmp::{max, min};

use crate::common::{
    coords::{VoxelCoords, WorldCoords},
    vertex::Vertex,
    voxel_material::VoxelMaterial,
    voxels::Voxel,
};
use std::collections::VecDeque;

use crate::marching_cubes;

#[derive(Component)]
pub struct ProceduralEntity {
    pub field_size: usize,

    // voxel field describing the geometry of the Entity
    // actual vertices will be determined by marching squares
    pub voxel_field: Vec<Voxel>,

    pub vertices: Vec<Vertex>,

    pub modification_count: usize,
    pub modification_threshold: usize,
}

impl ProceduralEntity {
    pub fn new(field_size: usize) -> Self {
        Self {
            // pos,
            field_size, // Initialize field size
            voxel_field: Vec::new(),
            vertices: Vec::new(),
            modification_count: 0,
            modification_threshold: 20, // Adjust based on your needs}
        }
    }

    pub fn generate_voxels(&mut self) {
        let start = Instant::now();
        self.voxel_field = vec![
            Voxel {
                value: -1.0,
                material: VoxelMaterial::AIR
            };
            self.field_size * self.field_size * self.field_size
        ];

        // sphere of radius 4 (centered at 10 10 10)
        for (i, v) in self.voxel_field.iter_mut().enumerate() {
            let center1 = VoxelCoords::new(
                self.field_size as u8 / 2 - 3,
                self.field_size as u8 / 2 - 3,
                self.field_size as u8 / 2 - 3,
            );
            let center2 = VoxelCoords::new(
                self.field_size as u8 / 2 + 3,
                self.field_size as u8 / 2 + 3,
                self.field_size as u8 / 2 + 3,
            );
            let curr = VoxelCoords {
                x: (i / (self.field_size * self.field_size)) as u8,
                y: ((i % (self.field_size * self.field_size)) / self.field_size) as u8,
                z: (i % self.field_size) as u8,
            };
            let dist = std::cmp::max(
                ordered_float::OrderedFloat(Self::sdf_sphere(curr, center1, 15.0)),
                ordered_float::OrderedFloat(Self::sdf_sphere(curr, center2, 15.0)),
            );
            v.value = -dist.into_inner();
        }
        let duration = start.elapsed();
        println!("entity voxel duration: {:?}", duration);
    }

    pub fn sdf_sphere(pos: VoxelCoords, center: VoxelCoords, radius: f32) -> f32 {
        let diff = center - pos;
        let dist = ((diff.x * diff.x + diff.y * diff.y + diff.z * diff.z) as f64).sqrt();
        dist as f32 - radius
    }

    pub fn generate_vertices(&mut self) {
        println!("field size: {}", self.field_size);
        let start = Instant::now();
        self.vertices.clear();
        marching_cubes::find_triangles(
            &mut self.vertices,
            &self.voxel_field,
            self.field_size as u32,
        );
        let duration = start.elapsed();
        println!("marching cubes: {:?}", duration);
    }

    pub fn minimize_field_size(&mut self) {
        let start = Instant::now();
        // Find the bounding box of positive voxels
        let mut min = VoxelCoords::new(
            self.field_size as u8,
            self.field_size as u8,
            self.field_size as u8,
        );
        let mut max = VoxelCoords::new(0, 0, 0);

        // Use SIMD to process chunks of voxels at once
        let chunk_size = 8; // Assuming we're using AVX (256-bit registers)
        let mut i = 0;
        while i < self.voxel_field.len() {
            let end = std::cmp::min(i + chunk_size, self.voxel_field.len());
            let mut mask = 0u8;

            unsafe {
                let mut values = _mm256_setzero_ps();
                for j in i..end {
                    let value = _mm256_set1_ps(self.voxel_field[j].value);
                    values = _mm256_max_ps(values, value);
                }
                mask = _mm256_movemask_ps(values) as u8;
            }

            for j in i..end {
                if (mask & (1 << (j - i))) != 0 {
                    let coords = self.index_to_coords(j);
                    min.x = min.x.min(coords.x);
                    min.y = min.y.min(coords.y);
                    min.z = min.z.min(coords.z);
                    max.x = max.x.max(coords.x);
                    max.y = max.y.max(coords.y);
                    max.z = max.z.max(coords.z);
                }
            }
            i += chunk_size;
        }
        println!("MIN: {:?}", min);
        println!("MAX: {:?}", max);
        println!("field size (before change): {:?}", self.field_size);

        // Ensure min and max are within bounds
        min.x = min.x.max(1);
        min.y = min.y.max(1);
        min.z = min.z.max(1);
        max.x = max.x.min(self.field_size as u8 - 2);
        max.y = max.y.min(self.field_size as u8 - 2);
        max.z = max.z.min(self.field_size as u8 - 2);

        // keep the largest
        let new_size = ((max.x - min.x).max(max.y - min.y).max(max.z - min.z) + 3) as usize; // +3 for padding on both sides
        if new_size < self.field_size {
            self.rebuild_field(min, new_size);
        }

        let duration = start.elapsed();
        println!("minimize_field_size duration: {:?}", duration);
    }

    fn index_to_coords(&self, index: usize) -> VoxelCoords {
        let z = (index % self.field_size) as u8;
        let y = ((index / self.field_size) % self.field_size) as u8;
        let x = (index / (self.field_size * self.field_size)) as u8;
        VoxelCoords::new(x, y, z)
    }

    fn rebuild_field(&mut self, min: VoxelCoords, new_size: usize) {
        let mut new_voxels = vec![
            Voxel {
                value: -1.0,
                material: VoxelMaterial::AIR,
            };
            new_size * new_size * new_size
        ];

        for x in 0..new_size {
            for y in 0..new_size {
                for z in 0..new_size {
                    let old_x = min.x as usize + x - 1;
                    let old_y = min.y as usize + y - 1;
                    let old_z = min.z as usize + z - 1;
                    if old_x < self.field_size && old_y < self.field_size && old_z < self.field_size
                    {
                        let old_index = old_z
                            + old_y * self.field_size
                            + old_x * self.field_size * self.field_size;
                        new_voxels[z + y * new_size + x * new_size * new_size] =
                            self.voxel_field[old_index];
                    }
                }
            }
        }

        self.voxel_field = new_voxels;
        self.field_size = new_size;
    }

    pub fn increase_field_size(&mut self) {
        let start = Instant::now();

        let mut min = VoxelCoords::new(u8::MAX, u8::MAX, u8::MAX);
        let mut max = VoxelCoords::new(0, 0, 0);
        let mut has_active_voxels = false;

        let chunk_size = 8; // AVX processes 8 floats at a time
        let mut i = 0;
        let threshold = unsafe { _mm256_set1_ps(0.0) };

        while i + chunk_size <= self.voxel_field.len() {
            unsafe {
                let values_ptr = self.voxel_field.as_ptr().add(i) as *const f32;
                let values = _mm256_load_ps(values_ptr); // Load 8 voxel values at once
                let mask = _mm256_cmp_ps(values, threshold, _CMP_GE_OQ);
                let bitmask = _mm256_movemask_ps(mask) as u8;

                if bitmask != 0 {
                    has_active_voxels = true;
                    for j in 0..chunk_size {
                        if (bitmask & (1 << j)) != 0 {
                            let coords = self.index_to_coords(i + j);
                            min.x = min.x.min(coords.x);
                            min.y = min.y.min(coords.y);
                            min.z = min.z.min(coords.z);
                            max.x = max.x.max(coords.x);
                            max.y = max.y.max(coords.y);
                            max.z = max.z.max(coords.z);
                        }
                    }
                }
            }
            i += chunk_size;
        }

        // Scalar cleanup for remaining elements (if any)
        while i < self.voxel_field.len() {
            if self.voxel_field[i].value >= 0.0 {
                has_active_voxels = true;
                let coords = self.index_to_coords(i);
                min.x = min.x.min(coords.x);
                min.y = min.y.min(coords.y);
                min.z = min.z.min(coords.z);
                max.x = max.x.max(coords.x);
                max.y = max.y.max(coords.y);
                max.z = max.z.max(coords.z);
            }
            i += 1;
        }

        if !has_active_voxels {
            min = VoxelCoords::new(0, 0, 0);
            max = VoxelCoords::new(
                (self.field_size - 1) as u8,
                (self.field_size - 1) as u8,
                (self.field_size - 1) as u8,
            );
        }

        let pad = 3;
        let new_size_x = max.x as usize - min.x as usize + pad;
        let new_size_y = max.y as usize - min.y as usize + pad;
        let new_size_z = max.z as usize - min.z as usize + pad;
        let new_size = new_size_x.max(new_size_y).max(new_size_z);

        if new_size > self.field_size {
            let old_size = self.field_size;
            let old_data = std::mem::take(&mut self.voxel_field);

            self.voxel_field = vec![
                Voxel {
                    value: -1.0,
                    material: VoxelMaterial::AIR,
                };
                new_size * new_size * new_size
            ];

            println!("Bounding box: min={:?}, max={:?}", min, max);
            println!("New size: {}", new_size);

            for (i, voxel) in old_data.into_iter().enumerate() {
                let old_coords = self.index_to_coords(i);
                let new_coords = old_coords;
                let new_index = new_coords.z as usize
                    + new_coords.y as usize * new_size
                    + new_coords.x as usize * new_size * new_size;
                if new_index < self.voxel_field.len() {
                    self.voxel_field[new_index] = voxel;
                }
            }

            self.field_size = new_size;
        }

        let duration = start.elapsed();
        println!("increase_field_size duration: {:?}", duration);
    }

    pub fn carve(
        &mut self,
        hit_position: Vec3,
        carve_speed: f32,
        carve_radius: usize,
    ) -> Vec<ProceduralEntity> {
        let converted = WorldCoords::from(hit_position);
        let voxel_coords = VoxelCoords::new(
            converted.x.into_inner() as u8,
            converted.y.into_inner() as u8,
            converted.z.into_inner() as u8,
        );

        for c in voxel_coords.iter_around(carve_radius) {
            let v = VoxelCoords::new(c.x as u8, c.y as u8, c.z as u8);
            let index = v.z as usize
                + v.y as usize * self.field_size
                + v.x as usize * self.field_size * self.field_size;
            if index >= self.voxel_field.len() {
                continue;
                // return Vec::new();
            }

            self.voxel_field[index].value -= carve_speed;
            self.modification_count += 1; // Increment here for each voxel change
        }

        self.extract_regions()
    }

    pub fn fill(
        &mut self,
        hit_position: Vec3,
        fill_speed: f32,
        fill_radius: usize,
    ) -> Vec<ProceduralEntity> {
        let converted = WorldCoords::from(hit_position);
        let voxel_coords = VoxelCoords::new(
            converted.x.into_inner() as u8,
            converted.y.into_inner() as u8,
            converted.z.into_inner() as u8,
        );

        let old_size = self.field_size;
        let mut should_increase = false;

        for c in voxel_coords.iter_around(fill_radius) {
            let v = VoxelCoords::new(c.x as u8, c.y as u8, c.z as u8);
            if v.x >= self.field_size as u8
                || v.y >= self.field_size as u8
                || v.z >= self.field_size as u8
            {
                should_increase = true;
                break;
            }
            let index = v.z as usize
                + v.y as usize * self.field_size
                + v.x as usize * self.field_size * self.field_size;
            if index < self.voxel_field.len() {
                self.voxel_field[index].value += fill_speed;
                self.voxel_field[index].value = self.voxel_field[index].value.min(1.0);
            }
        }

        if should_increase {
            self.increase_field_size();
        }

        vec![self.clone()]
    }

    /// Extract and return new entities for each connected region in the voxel field
    fn extract_regions(&self) -> Vec<ProceduralEntity> {
        let mut visited = vec![false; self.voxel_field.len()];
        let mut new_entities = Vec::new();

        for (i, voxel) in self.voxel_field.iter().enumerate() {
            if !visited[i] && voxel.value >= 0.0 {
                let mut region_voxels = vec![
                    Voxel {
                        value: -1.0,
                        material: VoxelMaterial::AIR,
                    };
                    self.voxel_field.len()
                ];
                for (j, v) in self.voxel_field.iter().enumerate() {
                    if v.value > 0.0 {
                        region_voxels[j].value = -0.1;
                    } else {
                        region_voxels[j].value = v.value;
                    }
                }
                let mut positive_voxel_count = 0;

                self.flood_fill_collect(
                    i,
                    &mut visited,
                    &mut region_voxels,
                    &mut positive_voxel_count,
                );

                if positive_voxel_count > 1 {
                    let new_entity = ProceduralEntity {
                        // pos: self.pos, // You might want to adjust the position based on the region's location
                        field_size: self.field_size, // Initialize field size
                        voxel_field: region_voxels,
                        vertices: Vec::new(),

                        modification_count: self.modification_count,
                        modification_threshold: self.modification_threshold,
                    };

                    new_entities.push(new_entity);
                }
            }
        }

        new_entities
    }

    /// Perform flood fill to collect all connected voxels into a new region voxel field
    fn flood_fill_collect(
        &self,
        start_index: usize,
        visited: &mut Vec<bool>,
        region_voxels: &mut Vec<Voxel>,
        positive_voxel_count: &mut usize,
    ) {
        let mut queue = VecDeque::new();
        queue.push_back(start_index);

        while let Some(index) = queue.pop_front() {
            if visited[index] {
                continue;
            }
            visited[index] = true;

            if self.voxel_field[index].value >= 0.0 {
                region_voxels[index] = self.voxel_field[index];
                *positive_voxel_count += 1;
            }

            let x = index / (self.field_size * self.field_size);
            let y = (index % (self.field_size * self.field_size)) / self.field_size;
            let z = index % self.field_size;

            let neighbors = [
                (x as isize + 1, y as isize, z as isize),
                (x as isize - 1, y as isize, z as isize),
                (x as isize, y as isize + 1, z as isize),
                (x as isize, y as isize - 1, z as isize),
                (x as isize, y as isize, z as isize + 1),
                (x as isize, y as isize, z as isize - 1),
            ];

            for (nx, ny, nz) in neighbors {
                if nx >= 0
                    && nx < self.field_size as isize
                    && ny >= 0
                    && ny < self.field_size as isize
                    && nz >= 0
                    && nz < self.field_size as isize
                {
                    let n_index = (nx as usize) * self.field_size * self.field_size
                        + (ny as usize) * self.field_size
                        + (nz as usize);
                    if !visited[n_index] && self.voxel_field[n_index].value >= 0.0 {
                        queue.push_back(n_index);
                    }
                }
            }
        }
    }
}

/*
 * Highly inneficient,
 * for now its okay
 */
impl Clone for ProceduralEntity {
    fn clone(&self) -> Self {
        let mut new_entity = ProceduralEntity::new(self.field_size);
        new_entity.modification_count = self.modification_count;
        new_entity.modification_threshold = self.modification_threshold;
        for i in 0..self.voxel_field.len() {
            new_entity.voxel_field.push(self.voxel_field[i]);
        }

        for v in self.vertices.iter() {
            new_entity.vertices.push(v.clone());
        }

        new_entity
    }
}
