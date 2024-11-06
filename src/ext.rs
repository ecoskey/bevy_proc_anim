use bevy_animation::prelude::AnimatableProperty;
use bevy_app::App;
use bevy_ecs::prelude::{Commands, EntityCommands};
use bevy_ecs::system::SystemId;

use crate::state::TickPoleMatching;
use crate::{
    component::Dynamics,
    state::{AnimHandle, AnimValuePlugin, DynamicsParams, DynamicsSource, DynamicsState},
    AnimValue,
};

pub trait DynamicsAppExt {
    fn init_animatable_type<T: AnimValue>(&mut self) -> &mut Self;
}

impl DynamicsAppExt for App {
    fn init_animatable_type<T: AnimValue>(&mut self) -> &mut Self {
        self.add_plugins(AnimValuePlugin::<T>::default())
    }
}

pub trait DynamicsCommandsExt {
    fn animate_value<T: AnimValue>(
        &mut self,
        initial: T,
        params: DynamicsParams,
        source: SystemId<(), T>,
    ) -> AnimHandle<T>;
}

impl<'w, 's> DynamicsCommandsExt for Commands<'w, 's> {
    fn animate_value<T: AnimValue>(
        &mut self,
        initial: T,
        params: DynamicsParams,
        source: SystemId<(), T>,
    ) -> AnimHandle<T> {
        let id = self
            .spawn((
                DynamicsState::new(initial, params),
                DynamicsSource(source),
                TickPoleMatching,
            ))
            .id();
        AnimHandle::new(id)
    }
}

pub trait DynamicsEntityCommandsExt {
    fn animate<P: AnimatableProperty<Property: AnimValue>>(
        &mut self,
        handle: AnimHandle<P::Property>,
    ) -> &mut Self;
}

impl<'a> DynamicsEntityCommandsExt for EntityCommands<'a> {
    fn animate<P: AnimatableProperty<Property: AnimValue>>(
        &mut self,
        handle: AnimHandle<P::Property>,
    ) -> &mut Self {
        self.entry::<Dynamics>()
            .or_insert(Dynamics::new())
            .and_modify(move |mut d| d.add::<P>(handle));
        self
    }
}
