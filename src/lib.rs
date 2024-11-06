use bevy_app::{App, Plugin, PreUpdate};
use bevy_color::{Laba, LinearRgba, Oklaba, Srgba, Xyza};
use bevy_ecs::prelude::Resource;
use bevy_ecs::schedule::{IntoSystemConfigs, IntoSystemSetConfigs};
use bevy_math::{Vec2, Vec3, Vec3A, Vec4, VectorSpace};
use component::apply_dynamics;
use state::{non_zero_delta, DynamicsSet, TickMode};

pub mod component;
mod ext;
pub mod props;
pub mod state;

pub use ext::*;

pub trait AnimValue: VectorSpace + Send + Sync + 'static {}
impl<T: VectorSpace + Send + Sync + 'static> AnimValue for T {}

#[derive(Resource)]
pub struct DefaultTickMode(pub TickMode);

#[derive(Default)]
pub struct DynamicsPlugin {
    pub default_tick_mode: TickMode,
}

impl Plugin for DynamicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DefaultTickMode(self.default_tick_mode))
            .configure_sets(
                PreUpdate,
                (
                    DynamicsSet::All.run_if(non_zero_delta),
                    DynamicsSet::Read.in_set(DynamicsSet::All),
                    DynamicsSet::Tick.in_set(DynamicsSet::All),
                    DynamicsSet::Write.in_set(DynamicsSet::All),
                    (DynamicsSet::Read, DynamicsSet::Tick, DynamicsSet::Write).chain(),
                ),
            )
            .add_systems(PreUpdate, apply_dynamics.in_set(DynamicsSet::Write));

        app.init_animatable_type::<f32>()
            .init_animatable_type::<Vec2>()
            .init_animatable_type::<Vec3>()
            .init_animatable_type::<Vec3A>()
            .init_animatable_type::<Vec4>()
            .init_animatable_type::<Xyza>()
            .init_animatable_type::<Srgba>()
            .init_animatable_type::<Oklaba>()
            .init_animatable_type::<LinearRgba>()
            .init_animatable_type::<Laba>();
    }
}
