use crate::common::voxel_material::VoxelMaterial;
use crate::common::{vertex::Vertex, voxels::Voxel};

use bevy::{math::Vec3, prelude::Color};

use crate::common::coords::*;

mod table;

// TODO: indexing, and spatial hierarchy for marching cubes
//
// some ideas :
// Reusing the vertices can save you memory and save time spent sending data to the GPU.
// It's really only worth the trouble if you have a large number of voxels.
// I've done this with marching cubes, but I'm not sure the same approach would work for tetrahedra.
//
// For marching cubes, the vertices lie along the edges. Each edge is shared by up to 4 cubes.
// The total number of unique edges in each dim is equal to size+1 because the two end
// voxels don't share their edges. So if you had nx*ny*nz voxels, you would have (nx+1)*(ny+1)*(nz+1) edges.
// This can be pre-allocated as an array. Each edge can potentially have a vertex.
// I used an integer index for tracking this, initialized to -1 to indicate empty/not yet calculated.
// Then iterate over each voxel and generate the edge points. Look up the edge for each vertex.
// If it's -1, add the vertex to the unique vertex array, assign the edge index to the current
// vertex array position, and add the index to the index list. If the edge is already assigned a value >= 0,
// then add that existing index to the index list.

#[derive(Clone, Copy)]
struct VertexNode {
    index: usize,
    pos: WorldCoords,
}

fn get_node_dn() -> VertexNode {
    VertexNode {
        index: 0,
        pos: WorldCoords::new(0.5, 0.0, 1.0),
    }
}
fn get_node_de() -> VertexNode {
    VertexNode {
        index: 1,
        pos: WorldCoords::new(1.0, 0.0, 0.5),
    }
}
fn get_node_ds() -> VertexNode {
    VertexNode {
        index: 2,
        pos: WorldCoords::new(0.5, 0.0, 0.0),
    }
}
fn get_node_dw() -> VertexNode {
    VertexNode {
        index: 3,
        pos: WorldCoords::new(0.0, 0.0, 0.5),
    }
}

fn get_node_un() -> VertexNode {
    VertexNode {
        index: 4,
        pos: WorldCoords::new(0.5, 1.0, 1.0),
    }
}
fn get_node_ue() -> VertexNode {
    VertexNode {
        index: 5,
        pos: WorldCoords::new(1.0, 1.0, 0.5),
    }
}
fn get_node_us() -> VertexNode {
    VertexNode {
        index: 6,
        pos: WorldCoords::new(0.5, 1.0, 0.0),
    }
}
fn get_node_uw() -> VertexNode {
    VertexNode {
        index: 7,
        pos: WorldCoords::new(0.0, 1.0, 0.5),
    }
}

fn get_node_nw() -> VertexNode {
    VertexNode {
        index: 8,
        pos: WorldCoords::new(0.0, 0.5, 1.0),
    }
}
fn get_node_ne() -> VertexNode {
    VertexNode {
        index: 9,
        pos: WorldCoords::new(1.0, 0.5, 1.0),
    }
}
fn get_node_se() -> VertexNode {
    VertexNode {
        index: 10,
        pos: WorldCoords::new(1.0, 0.5, 0.0),
    }
}
fn get_node_sw() -> VertexNode {
    VertexNode {
        index: 11,
        pos: WorldCoords::new(0.0, 0.5, 0.0),
    }
}

const NODES_POS_COUNT: usize = 12;
fn get_base_nodes() -> [VertexNode; NODES_POS_COUNT] {
    [
        get_node_dn(),
        get_node_de(),
        get_node_ds(),
        get_node_dw(),
        get_node_un(),
        get_node_ue(),
        get_node_us(),
        get_node_uw(),
        get_node_nw(),
        get_node_ne(),
        get_node_se(),
        get_node_sw(),
    ]
}

type Nodes = [Voxel; NODES_POS_COUNT];
type VoxelsBlock = [[[Voxel; 2]; 2]; 2];

fn get_voxel_with_field_size(field: &Vec<Voxel>, pos: VoxelCoords, field_size: u32) -> Voxel {
    let index = pos.z as u32 + pos.y as u32 * field_size + pos.x as u32 * field_size * field_size;
    field[index as usize]
}

fn get_voxels_for_vertex(
    field: &Vec<Voxel>,
    base_pos: VoxelCoords,
    field_size: u32,
) -> VoxelsBlock {
    let voxels: [[[Voxel; 2]; 2]; 2] = [
        [
            [
                get_voxel_with_field_size(field, base_pos + VoxelCoords::new(0, 0, 0), field_size),
                get_voxel_with_field_size(field, base_pos + VoxelCoords::new(0, 0, 1), field_size),
            ],
            [
                get_voxel_with_field_size(field, base_pos + VoxelCoords::new(0, 1, 0), field_size),
                get_voxel_with_field_size(field, base_pos + VoxelCoords::new(0, 1, 1), field_size),
            ],
        ],
        [
            [
                get_voxel_with_field_size(field, base_pos + VoxelCoords::new(1, 0, 0), field_size),
                get_voxel_with_field_size(field, base_pos + VoxelCoords::new(1, 0, 1), field_size),
            ],
            [
                get_voxel_with_field_size(field, base_pos + VoxelCoords::new(1, 1, 0), field_size),
                get_voxel_with_field_size(field, base_pos + VoxelCoords::new(1, 1, 1), field_size),
            ],
        ],
    ];
    voxels
}

fn chose_voxel_for_node(a: Voxel, b: Voxel) -> Voxel {
    if a.value < 0. {
        return Voxel {
            value: (-a.value) / (b.value - a.value),
            material: VoxelMaterial::AIR,
        };
    }
    if b.value < 0. {
        return Voxel {
            value: 1.0 - (-b.value) / (a.value - b.value),
            material: VoxelMaterial::AIR,
        };
    }
    Voxel {
        value: 0.,
        material: VoxelMaterial::AIR,
    }
}

fn get_vertex_nodes(voxels: VoxelsBlock) -> Nodes {
    let mut result: Nodes = [Voxel {
        value: 0.,
        material: VoxelMaterial::AIR,
    }; NODES_POS_COUNT];

    result[get_node_ds().index] = chose_voxel_for_node(voxels[0][0][0], voxels[1][0][0]);
    result[get_node_de().index] = chose_voxel_for_node(voxels[1][0][0], voxels[1][0][1]);
    result[get_node_dn().index] = chose_voxel_for_node(voxels[0][0][1], voxels[1][0][1]);
    result[get_node_dw().index] = chose_voxel_for_node(voxels[0][0][0], voxels[0][0][1]);

    result[get_node_ne().index] = chose_voxel_for_node(voxels[1][0][1], voxels[1][1][1]);
    result[get_node_nw().index] = chose_voxel_for_node(voxels[0][0][1], voxels[0][1][1]);
    result[get_node_se().index] = chose_voxel_for_node(voxels[1][0][0], voxels[1][1][0]);
    result[get_node_sw().index] = chose_voxel_for_node(voxels[0][0][0], voxels[0][1][0]);

    result[get_node_us().index] = chose_voxel_for_node(voxels[0][1][0], voxels[1][1][0]);
    result[get_node_ue().index] = chose_voxel_for_node(voxels[1][1][0], voxels[1][1][1]);
    result[get_node_un().index] = chose_voxel_for_node(voxels[0][1][1], voxels[1][1][1]);
    result[get_node_uw().index] = chose_voxel_for_node(voxels[0][1][0], voxels[0][1][1]);

    result
}

fn shift_node_pos(pos: WorldCoords, value: f32) -> WorldCoords {
    if pos.x.into_inner() == 0.5 {
        return WorldCoords::new(value, pos.y.into_inner(), pos.z.into_inner());
    }
    if pos.y.into_inner() == 0.5 {
        return WorldCoords::new(pos.x.into_inner(), value, pos.z.into_inner());
    }
    if pos.z.into_inner() == 0.5 {
        return WorldCoords::new(pos.x.into_inner(), pos.y.into_inner(), value);
    }

    panic!("failed to process pos {:?}", pos);
}

fn append_triangle(
    pos: VoxelCoords,
    vertices: &mut Vec<Vertex>,
    nodes: Nodes,
    a: VertexNode,
    b: VertexNode,
    c: VertexNode,
) {
    let a_v = nodes[a.index];
    let b_v = nodes[b.index];
    let c_v = nodes[c.index];

    if a_v.value < 0. || b_v.value < 0. || c_v.value < 0. {
        return;
    }

    let pos_vec = WorldCoords::new(pos.x as f32, pos.y as f32, pos.z as f32);

    let a_pos = shift_node_pos(a.pos, a_v.value) + pos_vec;
    let b_pos = shift_node_pos(b.pos, b_v.value) + pos_vec;
    let c_pos = shift_node_pos(c.pos, c_v.value) + pos_vec;

    let normal = (c_pos - a_pos).cross(b_pos - a_pos);
    let normal = Vec3::new(
        normal.x.into_inner(),
        normal.y.into_inner(),
        normal.z.into_inner(),
    );
    let normal = normal.normalize();

    // let sig3 = sigmoid(pos.y as f32, 20.0);
    let vc = Vertex {
        color: Color::srgb(0.3, 0.3, 0.3),
        normal,
        pos: c_pos,
        voxel_material: VoxelMaterial::AIR,
    };
    let vb = Vertex {
        color: Color::srgb(0.3, 0.3, 0.3),
        normal,
        pos: b_pos,
        voxel_material: VoxelMaterial::AIR,
    };
    let va = Vertex {
        color: Color::srgb(0.3, 0.3, 0.3),
        normal,
        pos: a_pos,
        voxel_material: VoxelMaterial::AIR,
    };

    vertices.push(vc);
    vertices.push(vb);
    vertices.push(va);
}

/// Applies marching cubes on a 3d field of Voxels
/// field_size represents the size of the field along one axis (only squared field allowed)
/// TODO: Allow using marching cubes on specific regions only
pub fn find_triangles(vertices: &mut Vec<Vertex>, field: &Vec<Voxel>, field_size: u32) {
    for x in 0..(field_size - 1) {
        for y in 0..(field_size - 1) {
            for z in 0..(field_size - 1) {
                let pos = VoxelCoords::new(x as u8, y as u8, z as u8);
                let voxels = get_voxels_for_vertex(field, pos, field_size);
                let nodes = get_vertex_nodes(voxels);

                let triangle_points = table::TABLE[table::get_index_by_voxels(voxels)];

                let mut triangle_offset = 0;

                let nodes_arr = get_base_nodes();

                while triangle_points[triangle_offset] != -1 {
                    let a = nodes_arr[triangle_points[triangle_offset] as usize];
                    let b = nodes_arr[triangle_points[triangle_offset + 1] as usize];
                    let c = nodes_arr[triangle_points[triangle_offset + 2] as usize];

                    append_triangle(pos, vertices, nodes, a, b, c);

                    triangle_offset += 3;
                }
            }
        }
    }
}
