use std::fmt::Debug;
use std::marker::PhantomData;

use bevy_animation::{animatable::Animatable, animation_curves::AnimatableProperty};
use bevy_ecs::component::Component;
use bevy_math::{Vec3, VectorSpace};
use bevy_reflect::{FromReflect, Reflect, Reflectable};
use bevy_transform::components::Transform;

use crate::AnimValue;

#[derive(Reflect)]
pub struct IdProperty<C>(#[reflect(ignore)] PhantomData<C>);

impl<C> Default for IdProperty<C> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<
        C: Component + Animatable + VectorSpace + FromReflect + Reflectable + Clone + Sync + Debug,
    > AnimatableProperty for IdProperty<C>
{
    type Component = C;

    type Property = C;

    fn get_mut(component: &mut Self::Component) -> Option<&mut Self::Property> {
        Some(component)
    }
}

#[derive(Reflect, Debug)]
pub struct TransformProperty;

impl AnimatableProperty for TransformProperty {
    type Component = Transform;

    type Property = Transform;

    fn get_mut(component: &mut Self::Component) -> Option<&mut Self::Property> {
        Some(component)
    }
}

#[derive(Reflect, Debug)]
pub struct TranslationProperty;

impl AnimatableProperty for TranslationProperty {
    type Component = Transform;

    type Property = Vec3;

    fn get_mut(component: &mut Self::Component) -> Option<&mut Self::Property> {
        Some(&mut component.translation)
    }
}

// pub struct RotationProperty;

#[derive(Reflect)]
pub struct ScaleProperty;

impl AnimatableProperty for ScaleProperty {
    type Component = Transform;

    type Property = Vec3;

    fn get_mut(component: &mut Self::Component) -> Option<&mut Self::Property> {
        Some(&mut component.scale)
    }
}
