use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::render::camera::Camera;

#[derive(Component, Default)]
pub struct DragOffset(Vec3);

#[derive(Component, Default)]
pub struct Cube;

pub fn on_drag_manipulate(
    drag: Trigger<Pointer<Drag>>,
    mut commands: Commands,
    mut transforms: Query<(&mut Transform, Option<&mut DragOffset>, &mut RigidBody)>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    kbd: Res<ButtonInput<KeyCode>>,
    window: Query<&Window>,
) {
    let Ok((mut transform, drag_offset, mut rigid_body)) = transforms.get_mut(drag.entity()) else {
        return;
    };

    // We don't want the object to be affected by other objects while dragging it
    *rigid_body = RigidBody::Kinematic;

    if kbd.pressed(KeyCode::ShiftLeft) {
        // Update the object's position based on the cursor and offset
        if let Some((camera, camera_transform)) = cameras.iter().next() {
            let window = window.get_single().unwrap();
            if let Some(cursor_pos) = window.cursor_position() {
                let ray = camera
                    .viewport_to_world(camera_transform, cursor_pos)
                    .unwrap();

                // Assuming the ground plane is at y = 0
                let ground_y = 0.0;

                // Compute intersection with the ground plane
                let t = (ground_y - ray.origin.y) / ray.direction.y;
                let intersection_point = ray.origin + ray.direction * t;

                // Calculate the offset if not already initialized
                let offset = drag_offset.map(|offset| offset.0).unwrap_or_else(|| {
                    let offset = transform.translation - intersection_point;
                    commands.entity(drag.entity()).insert(DragOffset(offset));
                    offset
                });

                // Update the object's position
                transform.translation.x = intersection_point.x + offset.x;
                transform.translation.z = intersection_point.z + offset.z;
                transform.translation.y = 0.5; // Keep the object slightly above the ground
            }
        }
    } else {
        // Rotate the object
        transform.rotate_y(drag.delta.x * 0.02);
        transform.rotate_x(drag.delta.y * 0.02);

        // Remove the offset when not translating
        if drag_offset.is_some() {
            commands.entity(drag.entity()).remove::<DragOffset>();
        }
    }
}

pub fn on_drag_end(
    drag_end: Trigger<Pointer<DragEnd>>,
    mut query: Query<&mut RigidBody, With<Cube>>,
) {
    // Get the entity that was dropped
    let dropped_entity = drag_end.entity();

    // Check if the dropped entity is in the query and modify its RigidBody
    if let Ok(mut rigid_body) = query.get_mut(dropped_entity) {
        if *rigid_body != RigidBody::Dynamic {
            *rigid_body = RigidBody::Dynamic;
        }
    }
}
