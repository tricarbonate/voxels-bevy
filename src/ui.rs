use crate::camera::*;
use crate::common::coords::*;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

/// Main UI (mostly for debugging purposes)
pub fn ui_main_system(
    mut contexts: EguiContexts,
    camera_transform_q: Query<&Transform, With<FirstPersonState>>,
) {
    let camera_pos = camera_transform_q.single().translation;
    egui::Window::new("World Info").show(contexts.ctx_mut(), |ui| {
        ui.label("Camera:");
        ui.label(format!("x: {}", camera_pos.x));
        ui.label(format!("y: {}", camera_pos.y));
        ui.label(format!("z: {}", camera_pos.z));
        let w_coords = WorldCoords::from(camera_pos);
        let vpos = VoxelCoords::from(w_coords);
        ui.label(format!("vx: {}", vpos.x));
        ui.label(format!("vy: {}", vpos.y));
        ui.label(format!("vz: {}", vpos.z));

        // let chunk_coords = ChunkCoords::from_vec(camera_pos);
        // let world_coords = WorldCoords::from(camera_pos);

        // NOTE: disabling printing temperature as it would lock the current chunk
        // let temp = match chunks.0.get_mut(&chunk_coords) {
        // Some(c) => c.temp_at(world_coords),
        //     _ => 0.0,
        // };
        // ui.label(format!("temperature: {}", temp));
    });
}
