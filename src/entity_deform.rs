use avian3d::prelude::*;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::entity_mesh::EntityMeshComponent;
use crate::resources::{FillMode, ProcEntities, RayMeshHits};

pub fn entity_deform_system(
    mut proc_entities: ResMut<ProcEntities>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ray_hits: ResMut<RayMeshHits>,
    transform_q: Query<(&Transform, &LinearVelocity, &AngularVelocity)>,
    fill_mode: Res<FillMode>,
) {
    // handle next ray hit (FIFO order)
    if let Some(hit) = ray_hits.0.pop_front() {
        // check to see if the hit is actually targeting a registered entity
        let e = if let Some(en) = proc_entities.0.get(&hit.0) {
            Arc::clone(&en)
        } else {
            return;
        };

        let t = transform_q.get(hit.0).unwrap().0;
        let lv = transform_q.get(hit.0).unwrap().1;
        let av = transform_q.get(hit.0).unwrap().2;

        // hit point in the entity local space
        let local_hit_point =
            t.rotation.inverse() * (hit.1.point - t.translation) * (1.0 / t.scale);

        // fill or carve the entity and regenerate its vertices
        let mut new_entities = {
            let mut entity = e.lock().unwrap();
            if fill_mode.0 {
                entity.fill(local_hit_point, 0.3, 2)
            } else {
                entity.carve(local_hit_point, 0.2, 2)
            }
        };

        for ent in new_entities.iter_mut() {
            if ent.modification_count >= ent.modification_threshold {
                ent.minimize_field_size();
                ent.modification_count = 0; // Reset modification count
            }
            ent.generate_vertices();
        }

        // despawn the entity and respawn it entirely
        commands.entity(hit.0).despawn_recursive();

        for ent in new_entities.iter_mut() {
            let ac_mtx_entity = Arc::new(Mutex::new(ent.clone()));
            let new_en = EntityMeshComponent::respawn(
                &mut commands,
                &mut meshes,
                &mut materials,
                Arc::clone(&ac_mtx_entity),
                t.clone(),
                lv.clone(),
                av.clone(),
            );

            // replace the entity in proc_entities
            // we insert first so that the reference count of ProceduralEntity remains positive
            proc_entities.0.insert(new_en, Arc::clone(&ac_mtx_entity));
        }
        proc_entities.0.remove(&hit.0);
    }
}
