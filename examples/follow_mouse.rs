use bevy::{prelude::*, window::PrimaryWindow};
use bevy_proc_anim::{
    props::TranslationProperty, state::DynamicsParams, DynamicsCommandsExt,
    DynamicsEntityCommandsExt, DynamicsPlugin,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DynamicsPlugin::default()))
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    let track_mouse_id = commands.register_system(track_mouse);
    let pos_handle = commands.animate_value(
        Vec3::ZERO,
        DynamicsParams::new(3.0, 0.5, 2.0),
        track_mouse_id,
    );
    commands.spawn(Camera2d);
    commands
        .spawn((
            Mesh2d(meshes.add(Circle::new(100.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::WHITE))),
        ))
        .animate::<TranslationProperty>(pos_handle);
}

fn track_mouse(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
) -> Vec3 {
    window
        .cursor_position()
        .and_then(|viewport_pos| camera.0.viewport_to_world(camera.1, viewport_pos).ok())
        .map(|ray| ray.origin.truncate())
        .map(|cursor_pos| Vec3::new(cursor_pos.x, cursor_pos.y, 0.0))
        .unwrap_or(Vec3::ZERO)
}
