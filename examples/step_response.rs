use bevy::prelude::*;
use bevy_proc_anim::{
    props::TranslationProperty, Dynamic, DynamicsParams, DynamicsPlugin, DynamicsTickMode,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            DynamicsPlugin::<TranslationProperty>::new(DynamicsTickMode::PoleMatching),
        ))
        .add_systems(Startup, (setup_squares, setup_camera))
        .add_systems(Update, update_dynamics)
        .insert_resource(DynamicsTimer(
            Timer::from_seconds(1.0, TimerMode::Repeating),
            150.0,
        ))
        .run();
}

fn setup_squares(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    const NUM_SQUARES: u32 = 40;

    let mesh = meshes.add(Rectangle::new(20.0, 20.0));

    for i in 1..=NUM_SQUARES {
        let color = Color::hsl(360. * i as f32 / NUM_SQUARES as f32, 0.95, 0.7);
        let pos = Vec3::new(
            1000.0 * ((i as f32 - 0.5) / NUM_SQUARES as f32 - 0.5),
            0.0,
            0.0,
        );
        commands.spawn((
            Mesh2d(mesh.clone()),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_translation(pos),
            Dynamic::<TranslationProperty>::new(
                pos,
                DynamicsParams::new(i as f32 / 10.0, 0.5, 2.0),
            ),
        ));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Resource)]
struct DynamicsTimer(Timer, f32);

fn update_dynamics(
    mut timer: ResMut<DynamicsTimer>,
    mut dynamics: Query<&mut Dynamic<TranslationProperty>>,
    time: Res<Time>,
) {
    if timer.0.tick(time.delta()).times_finished_this_tick() >= 1 {
        timer.1 = -timer.1;

        for mut dynamic in &mut dynamics {
            dynamic.target.y = timer.1;
        }
    }
}
