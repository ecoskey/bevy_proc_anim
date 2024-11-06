use bevy::{prelude::*, window::PrimaryWindow};
use bevy_proc_anim::{
    props::TranslationProperty, AnimValuePlugin, DynamicsParams, DynamicsState, TickMode,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            AnimValuePlugin::<TranslationProperty>::new(DynamicsTickMode::PoleMatching),
        ))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, update_dynamics)
        .run();
}

fn setup_scene(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(100.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::WHITE))),
        DynamicsState::<TranslationProperty>::new(Vec3::ZERO, DynamicsParams::new(3.0, 0.5, 2.0)),
    ));
}

fn update_dynamics(
    mut circle: Single<&mut DynamicsState<TranslationProperty>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    if let Some(cursor_pos) = window
        .cursor_position()
        .and_then(|viewport_pos| camera.0.viewport_to_world(camera.1, viewport_pos).ok())
        .map(|ray| ray.origin.truncate())
    {
        circle.target = Vec3::new(cursor_pos.x, cursor_pos.y, 0.0);
    }
}
