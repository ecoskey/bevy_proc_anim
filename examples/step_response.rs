use bevy::prelude::*;
use bevy_proc_anim::{
    prop, state::DynamicsParams, DynamicsCommandsExt, DynamicsEntityCommandsExt, DynamicsPlugin,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DynamicsPlugin::default()))
        .add_systems(Startup, (setup_squares, setup_camera))
        .add_systems(Update, tick_timer)
        .insert_resource(DynamicsTimer(
            Timer::from_seconds(1.0, TimerMode::Repeating),
            false,
        ))
        .run();
}

fn setup_squares(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bird_sprite = asset_server.load::<Image>("bevy_bird_dark.png");

    const NUM_SQUARES: u32 = 40;

    let sat_id = commands.register_system(update_saturation);
    let transform_id = commands.register_system(update_transform);

    for i in 1..=NUM_SQUARES {
        let color = Color::hsl(360. * i as f32 / NUM_SQUARES as f32, 0.95, 0.7);
        let pos = Vec3::new(
            1000.0 * ((i as f32 - 0.5) / NUM_SQUARES as f32 - 0.5),
            0.0,
            0.0,
        );

        let params = DynamicsParams::new(i as f32 / 10.0, 0.5, 2.0);

        let sat_handle = commands.animate_value(0.5, params, sat_id);
        let tf_handle = commands.animate_value(0.0, params, transform_id);

        commands
            .spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(25.0)),
                    image: bird_sprite.clone(),
                    ..Default::default()
                },
                Transform::from_translation(pos),
            ))
            .animate::<SpriteSaturationProperty>(sat_handle)
            .animate::<TranslationYProperty>(tf_handle);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

prop!(
    pub struct TranslationYProperty,
    Transform,
    f32,
    |tf| &mut tf.translation.y
);

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
struct DynamicsTimer(Timer, bool);

fn tick_timer(mut timer: ResMut<DynamicsTimer>, time: Res<Time>) {
    if timer.0.tick(time.delta()).times_finished_this_tick() >= 1 {
        timer.1 = !timer.1;
    }
}

fn update_saturation(timer: Res<DynamicsTimer>) -> f32 {
    timer.1 as u32 as f32
}

fn update_transform(timer: Res<DynamicsTimer>) -> f32 {
    if timer.1 {
        150.0
    } else {
        -150.0
    }
}
