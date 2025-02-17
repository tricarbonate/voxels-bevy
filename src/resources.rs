use bevy::picking::mesh_picking::ray_cast::RayMeshHit;
use bevy::prelude::*;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use crate::procedural_entity::*;

/// All procedurally generated entities
/// Because these entities are deformable, we need to regenerate their voxels and corresponding
/// bevy meshes quite often, this is why we use an Arc<Mutex<T>> to avoid frequent cloning
#[derive(Resource, Default)]
pub struct ProcEntities(pub BTreeMap<Entity, Arc<Mutex<ProceduralEntity>>>);

/// List of all ray hits that need to be handled
/// We store them in this VecQueue because some ray hits may cause a lot of work and may need
/// to be paralelized, so we want them on different systems
#[derive(Resource, Default)]
pub struct RayMeshHits(pub VecDeque<(Entity, RayMeshHit)>);

#[derive(Resource, Default)]
pub struct FillMode(pub bool); // true for fill, false for carve:
