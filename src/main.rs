// #![feature(portable_simd)]
#![feature(stdarch_x86_avx512)]

mod camera;
use crate::resources::FillMode;
mod common;
mod entity_deform;
mod entity_mesh;
mod marching_cubes;
mod observers;
mod procedural_entity;
mod resources;
mod ui;
use crate::entity_mesh::EntityMeshComponent;
use avian3d::prelude::*;
use bevy::prelude::*;
use camera::*;
use entity_deform::*;
use observers::*;
use procedural_entity::*;
use resources::*;
use std::sync::Arc;
use std::sync::Mutex;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(), //.set(PhysicsInterpolationPlugin::interpolate_all()),
            MeshPickingPlugin,
            // enable for physics debug rendering
            // PhysicsDebugPlugin::default(),
        ))
        // .add_plugins(EguiPlugin)
        .insert_resource(ProcEntities::default())
        .insert_resource(resources::RayMeshHits::default())
        .insert_resource(FillMode::default())
        .add_systems(Startup, setup) // Add a basic 3D scene setup
        .add_systems(Startup, spawn_camera)
        .add_systems(
            Update,
            (
                handle_camera.run_if(any_with_component::<FirstPersonState>),
                entity_deform_system,
            )
                .chain(),
        )
        .add_systems(Update, grab_mouse)
        .add_systems(Update, toggle_fill_mode)
        // .add_systems(Update, cursor_recenter)
        // .add_systems(Update, ui_main_system)
        .run();
}

pub fn toggle_fill_mode(
    mut fill_mode: ResMut<FillMode>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        fill_mode.0 = !fill_mode.0;
        println!("Fill mode: {}", if fill_mode.0 { "ON" } else { "OFF" });
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut proc_entities: ResMut<ProcEntities>,
) {
    commands
        .spawn((
            Text::new("Click Me to get a box\nDrag cubes to rotate"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(12.0),
                left: Val::Percent(12.0),
                ..default()
            },
        ))
        .observe(on_click_spawn_cube)
        .observe(
            |out: Trigger<Pointer<Out>>, mut texts: Query<&mut TextColor>| {
                let mut text_color = texts.get_mut(out.entity()).unwrap();
                text_color.0 = Color::WHITE;
            },
        )
        .observe(
            |over: Trigger<Pointer<Over>>, mut texts: Query<&mut TextColor>| {
                let mut color = texts.get_mut(over.entity()).unwrap();
                color.0 = bevy::color::palettes::tailwind::CYAN_400.into();
            },
        );

    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(100.0, 0.1),
        Mesh3d(meshes.add(Cylinder::new(50.0, 0.1))),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));

    commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cuboid(0.5, 0.5, 0.5),
            AngularVelocity(Vec3::new(2.5, 3.5, 1.5)),
            Mesh3d(meshes.add(Cuboid::from_length(0.5))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            Transform::from_xyz(0.0, 4.0, 0.0),
        ))
        .insert(Cube)
        .observe(on_drag_manipulate)
        .observe(on_drag_end);

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 8.0, 0.0),
    ));
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(30.0, 8.0, 30.0),
    ));

    let entity = Arc::new(Mutex::new(ProceduralEntity::new(40)));
    {
        let mut entity_guard = entity.lock().unwrap();
        entity_guard.generate_voxels();
        entity_guard.minimize_field_size();
        entity_guard.generate_vertices();
    }
    let id = EntityMeshComponent::spawn(
        &mut commands,
        &mut meshes,
        &mut materials,
        Arc::clone(&entity),
    );
    proc_entities.0.insert(id, entity);
}

struct CubeCount(usize);
impl Default for CubeCount {
    fn default() -> CubeCount {
        return CubeCount(1);
    }
}
fn on_click_spawn_cube(
    _click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut proc_entities: ResMut<ProcEntities>,
    mut num: Local<CubeCount>,
) {
    // spawn cube
    commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cuboid(0.5, 0.5, 0.5),
            Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            Transform::from_xyz(0.0, 0.25 + 0.55 * (*num).0 as f32, 0.0),
        ))
        .insert(Cube)
        .observe(on_drag_manipulate)
        .observe(on_drag_end);
    (*num).0 += 1;

    let entity = Arc::new(Mutex::new(ProceduralEntity::new(40)));
    entity.lock().unwrap().generate_voxels();
    entity.lock().unwrap().generate_vertices();
    let id = EntityMeshComponent::spawn(
        &mut commands,
        &mut meshes,
        &mut materials,
        Arc::clone(&entity),
    );
    proc_entities.0.insert(id, entity);
}
