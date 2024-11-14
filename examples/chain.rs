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
    commands.spawn(Camera2d);

    let mesh = meshes.add(Circle::new(5.0));
    let material = materials.add(ColorMaterial::from_color(Color::WHITE));
    let params = DynamicsParams::new(10.0, 1.0, 0.0);

    let track_mouse = commands.register_system(track_mouse);
    let track_mouse_handle = commands.animate_value(Vec3::ZERO, params, track_mouse);

    let first = commands
        .spawn((Mesh2d(mesh.clone()), MeshMaterial2d(material.clone())))
        .animate::<TranslationProperty>(track_mouse_handle)
        .id();

    let mut prev = first;
    const FOLLOWERS: u32 = 10;
    const MARGIN: f32 = 20.0;

    for _ in 0..FOLLOWERS {
        let curr = commands
            .spawn((Mesh2d(mesh.clone()), MeshMaterial2d(material.clone())))
            .id();
        let follow_previous = commands.register_system(follow_previous(prev, curr, MARGIN));
        let follow_previous_handle = commands.animate_value(Vec3::ZERO, params, follow_previous);
        commands
            .entity(curr)
            .animate::<TranslationProperty>(follow_previous_handle);
        prev = curr;
    }
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

fn follow_previous(
    previous: Entity,
    current: Entity,
    margin: f32,
) -> impl System<In = (), Out = Vec3> {
    IntoSystem::into_system(move |pos: Query<&Transform>| {
        let [prev_tf, curr_tf] = pos.get_many([previous, current]).unwrap();
        let diff = curr_tf.translation - prev_tf.translation;
        if diff == Vec3::ZERO {
            curr_tf.translation
        } else {
            prev_tf.translation + diff.normalize() * margin
        }
    })
}
