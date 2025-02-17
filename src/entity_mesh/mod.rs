pub mod entity_mesh;

use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::utils::Instant;

use crate::common::vertex::Vertex;
use crate::common::voxel_material::VoxelMaterial;
use crate::procedural_entity::ProceduralEntity;
use crate::Cube;
use std::sync::{Arc, Mutex};

#[derive(Component, Clone, Copy)]
pub struct EntityMeshComponent;

impl EntityMeshComponent {
    pub fn respawn(
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        entity: Arc<Mutex<ProceduralEntity>>,
        transform: Transform,
        lv: LinearVelocity,
        av: AngularVelocity,
    ) -> Entity {
        // we cannot call entity.lock() multiple times during the same function call
        let vertices = entity.lock().unwrap().vertices.clone();

        let mesh = Self::generate_mesh(vertices);

        let id = commands
            .spawn((
                RigidBody::Dynamic,
                Collider::trimesh_from_mesh(&mesh).unwrap(),
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(124, 144, 255),
                    ..default()
                })),
                transform,
                lv,
                av,
            ))
            .insert(EntityMeshComponent)
            .insert(Cube)
            .observe(crate::observers::on_drag_manipulate)
            .id();

        return id;
    }

    pub fn spawn(
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        entity: Arc<Mutex<ProceduralEntity>>,
    ) -> Entity {
        // we cannot call entity.lock() multiple times during the same function call
        let vertices = entity.lock().unwrap().vertices.clone();

        let mesh = Self::generate_mesh(vertices);

        let id = commands
            .spawn((
                RigidBody::Dynamic,
                Collider::trimesh_from_mesh(&mesh).unwrap(),
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(124, 144, 255),
                    ..default()
                })),
                Transform::default()
                    .with_translation(Vec3::default())
                    .with_scale(Vec3::new(0.2, 0.2, 0.2)),
            ))
            .insert(EntityMeshComponent)
            .insert(Cube)
            .observe(crate::observers::on_drag_manipulate)
            .id();

        return id;
    }

    pub fn generate_mesh(vertices: Vec<Vertex>) -> Mesh {
        let mut indices_vec = Vec::new();

        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut colors: Vec<[f32; 4]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        for vertex in vertices.iter() {
            indices_vec.push(positions.len() as u32);
            positions.push(vertex.pos.into_inners_arr());
            normals.push(vertex.normal.into());
            let c = if vertex.voxel_material == VoxelMaterial::STONE {
                [0.4, 0.4, 0.4, 1.0]
            } else {
                [0.6, 0.5, 0.2, 1.0]
            };
            colors.push(c);
            uvs.push([1., 1.]);
        }

        let indices = mesh::Indices::U32(indices_vec);

        // TODO: Maybe use a triangle strip instead ?
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );
        mesh.insert_indices(indices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

        mesh
    }
}
