// use avian3d::prelude::*;
// use bevy::asset::RenderAssetUsages;
// use bevy::prelude::*;
// use bevy::render::mesh::{self, PrimitiveTopology};
// use bevy::utils::Instant;
//
// // use bevy_mod_raycast::RaycastMesh;
//
// use crate::common::vertex::Vertex;
// use crate::common::voxel_material::VoxelMaterial;
// use crate::procedural_entity::ProceduralEntity;
// // use crate::plugins::physics::filter_tags::FilterTag;
// use std::sync::{Arc, Mutex};
//
// #[derive(Component, Clone, Copy)]
// pub struct EntityMeshComponent;
//
// impl EntityMeshComponent {
//     pub fn spawn(
//         commands: &mut Commands,
//         meshes: &mut Assets<Mesh>,
//         materials: &mut Assets<StandardMaterial>,
//         entity: Arc<Mutex<ProceduralEntity>>,
//     ) -> Entity {
//         let start = Instant::now();
//         // TODO: faster way to generate trimesh ??
//         let mut ind_pos: Vec<Vec3> = Vec::new();
//         let mut ind: Vec<[u32; 3]> = Vec::new();
//         for v in &entity.lock().unwrap().vertices {
//             ind_pos.push(Vec3 {
//                 x: v.pos.x.into_inner(),
//                 y: v.pos.y.into_inner(),
//                 z: v.pos.z.into_inner(),
//             });
//         }
//         for i in (0..entity.lock().unwrap().vertices.len() - 3).step_by(3) {
//             ind.push([i as u32, i as u32 + 1, i as u32 + 2]);
//         }
//
//         let mut duration = start.elapsed();
//         println!("data copy time {:?}", duration);
//         let start = Instant::now();
//
//         let collider = Collider::trimesh(ind_pos, ind);
//         duration = start.elapsed();
//         println!("collider creation time {:?}", duration);
//
//         let start = Instant::now();
//         // we cannot call entity.lock() multiple times during the same function call
//         let vertices = entity.lock().unwrap().vertices.clone();
//         let scale = entity.lock().unwrap().scale;
//         let pos = entity.lock().unwrap().pos;
//         duration = start.elapsed();
//         println!("vertices cloning time: {:?}", duration);
//
//         let start = Instant::now();
//         let id = commands
//             .spawn((
//                 Mesh3d(meshes.add(Self::generate_mesh(vertices))),
//                 MeshMaterial3d(materials.add(StandardMaterial {
//                     base_color: Color::rgb(0.5, 0.5, 0.5),
//                     cull_mode: None,
//                     perceptual_roughness: 0.6,
//                     ..default()
//                 })),
//             ))
//             .insert(EntityMeshComponent)
//             .insert(collider)
//             .insert(RigidBody::Dynamic)
//             // allow filtering the intersections and contact points on the chunks
//             .id();
//
//         duration = start.elapsed();
//         println!("actual spawning time {:?}", duration);
//
//         return id;
//     }
//
//     pub fn generate_mesh(vertices: Vec<Vertex>) -> Mesh {
//         let mut indices_vec = Vec::new();
//
//         let mut positions: Vec<[f32; 3]> = Vec::new();
//         let mut normals: Vec<[f32; 3]> = Vec::new();
//         let mut colors: Vec<[f32; 4]> = Vec::new();
//         let mut uvs: Vec<[f32; 2]> = Vec::new();
//         for vertex in vertices.iter() {
//             indices_vec.push(positions.len() as u32);
//             positions.push(vertex.pos.into_inners_arr());
//             normals.push(vertex.normal.into());
//             let c = if vertex.voxel_material == VoxelMaterial::STONE {
//                 [0.4, 0.4, 0.4, 1.0]
//             } else {
//                 [0.6, 0.5, 0.2, 1.0]
//             };
//             colors.push(c);
//             uvs.push([1., 1.]);
//         }
//
//         let indices = mesh::Indices::U32(indices_vec);
//
//         // TODO: Maybe use a triangle strip instead ?
//         let mut mesh = Mesh::new(
//             PrimitiveTopology::TriangleList,
//             RenderAssetUsages::MAIN_WORLD,
//         );
//         mesh.insert_indices(indices);
//         mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
//         mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
//         mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
//         mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
//
//         mesh
//     }
// }
