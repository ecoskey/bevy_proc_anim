use std::fmt::Debug;
use std::marker::PhantomData;

use bevy_animation::{animatable::Animatable, animation_curves::AnimatableProperty};
use bevy_ecs::component::Component;
use bevy_math::{Vec3, VectorSpace};
use bevy_reflect::{FromReflect, Reflect, Reflectable};
use bevy_transform::components::Transform;

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

#[macro_export]
macro_rules! prop {
    ($pub:vis struct $name:ident, $component: ident, $prop: ident, $op: expr) => {
        #[derive(Reflect, Debug)]
        $pub struct $name;

        impl AnimatableProperty for $name {
            type Component = $component;
            type Property = $prop;

            fn get_mut(component: &mut Self::Component) -> Option<&mut Self::Property> {
                let op: fn(&mut Self::Component) -> &mut Self::Property = $op;
                Some((op)(component))
            }
        }
    };
}

prop!(pub struct TranslationProperty, Transform, Vec3, |tf| &mut tf.translation);
prop!(pub struct ScaleProperty, Transform, Vec3, |tf| &mut tf.scale);
