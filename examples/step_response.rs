use bevy::prelude::*;
use bevy_proc_anim::{
    props::TranslationProperty, Dynamic, DynamicsParams, DynamicsPlugin, DynamicsTickMode,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            DynamicsPlugin::<TranslationProperty>::new(DynamicsTickMode::PoleMatching),
            DynamicsPlugin::<SpriteSaturationProperty>::new(DynamicsTickMode::PoleMatching),
        ))
        .add_systems(Startup, (setup_squares, setup_camera))
        .add_systems(Update, update_dynamics)
        .insert_resource(DynamicsTimer(
            Timer::from_seconds(1.0, TimerMode::Repeating),
            150.0,
        ))
        .run();
}

fn setup_squares(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bird_sprite = asset_server.load::<Image>("bevy_bird_dark.png");

    const NUM_SQUARES: u32 = 40;
    for i in 1..=NUM_SQUARES {
        let color = Color::hsl(360. * i as f32 / NUM_SQUARES as f32, 0.95, 0.7);
        let pos = Vec3::new(
            1000.0 * ((i as f32 - 0.5) / NUM_SQUARES as f32 - 0.5),
            0.0,
            0.0,
        );

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(25.0)),
                    ..Default::default()
                },
                transform: Transform::from_translation(pos),
                texture: bird_sprite.clone(),
                ..Default::default()
            },
            Dynamic::<TranslationProperty>::new(
                pos,
                DynamicsParams::new(i as f32 / 10.0, 0.5, 2.0),
            ),
            Dynamic::<SpriteSaturationProperty>::new(
                0.5,
                DynamicsParams::new(i as f32 / 10.0, 0.5, 2.0),
            ),
        ));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Reflect)]
struct SpriteSaturationProperty;

impl AnimatableProperty for SpriteSaturationProperty {
    type Component = Sprite;

    type Property = f32;

    fn get_mut(component: &mut Self::Component) -> Option<&mut Self::Property> {
        match component.color {
            Color::Hsla(ref mut hsla) => Some(&mut hsla.saturation),
            _ => None,
        }
    }
}

#[derive(Resource)]
struct DynamicsTimer(Timer, f32);

fn update_dynamics(
    mut timer: ResMut<DynamicsTimer>,
    mut pos_dynamics: Query<&mut Dynamic<TranslationProperty>>,
    mut color_dynamics: Query<&mut Dynamic<SpriteSaturationProperty>>,
    time: Res<Time>,
) {
    if timer.0.tick(time.delta()).times_finished_this_tick() >= 1 {
        timer.1 = -timer.1;

        let saturation = if timer.1 > 0.0 { 1.0 } else { 0.0 };

        for mut pos_dynamic in &mut pos_dynamics {
            pos_dynamic.target.y = timer.1;
        }

        for mut color_dynamic in &mut color_dynamics {
            color_dynamic.target = saturation;
        }
    }
}
