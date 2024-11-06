use bevy_ecs::{
    component::Component,
    entity::Entity,
    prelude::{Query, Without},
    world::{EntityMutExcept, EntityRef},
};
use bevy_utils::TypeIdMap;
use std::{any::TypeId, ops::DerefMut};

use bevy_animation::prelude::AnimatableProperty;

use crate::{
    state::{AnimHandle, DynamicsState},
    AnimValue,
};

#[derive(Component)]
pub struct Dynamics {
    props: TypeIdMap<Box<dyn PropertyUpdate>>,
}

impl Dynamics {
    pub(crate) fn new() -> Self {
        Self {
            props: Default::default(),
        }
    }

    pub fn add<P: AnimatableProperty<Property: AnimValue>>(
        &mut self,
        source: AnimHandle<P::Property>,
    ) {
        self.props
            .insert(TypeId::of::<P>(), Box::new(PropertyWrapper::<P> { source }));
    }

    pub fn remove<P: AnimatableProperty<Property: AnimValue>>(&mut self) {
        self.props.remove(&TypeId::of::<P>());
    }

    fn apply(
        &self,
        sources: &Query<EntityRef, Without<Dynamics>>,
        mut destination: EntityMutExcept<Dynamics>,
    ) {
        self.props.values().for_each(|prop| {
            if let Ok(source) = sources.get(prop.source()) {
                prop.apply(source, &mut destination);
            }
        })
    }
}

trait PropertyUpdate: Send + Sync + 'static {
    fn source(&self) -> Entity;
    fn apply(&self, source: EntityRef, destination: &mut EntityMutExcept<Dynamics>);
}

struct PropertyWrapper<P: AnimatableProperty<Property: AnimValue>> {
    source: AnimHandle<P::Property>,
}

impl<P: AnimatableProperty<Property: AnimValue>> PropertyUpdate for PropertyWrapper<P> {
    fn source(&self) -> Entity {
        self.source.entity
    }

    fn apply(&self, source: EntityRef, destination: &mut EntityMutExcept<Dynamics>) {
        let Some(state) = source.get::<DynamicsState<P::Property>>() else {
            return;
        };
        let Some(mut component) = destination.get_mut::<P::Component>() else {
            return;
        };
        let Some(prop) = P::get_mut(component.deref_mut()) else {
            return;
        };

        *prop = state.value();
    }
}

pub(super) fn apply_dynamics(
    mut dynamics: Query<(EntityMutExcept<Dynamics>, &Dynamics)>,
    sources: Query<EntityRef, Without<Dynamics>>,
) {
    for (entity, dynamics) in &mut dynamics {
        dynamics.apply(&sources, entity);
    }
}
