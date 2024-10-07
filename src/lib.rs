pub mod props;

use std::{f32::consts::PI, fmt::Debug, marker::PhantomData, ops::DerefMut};

use bevy_animation::animation_curves::AnimatableProperty;
use bevy_app::App;
use bevy_app::{Plugin, PreUpdate};
use bevy_ecs::{
    component::Component,
    system::{Query, Res},
};
use bevy_math::{
    ops::{self, FloatPow},
    VectorSpace,
};
use bevy_time::Time;

#[derive(Component)]
pub struct Dynamic<P: AnimatableProperty<Property: VectorSpace>> {
    pub target: P::Property,
    pub params: DynamicsParams,

    prev_target: P::Property,
    current: P::Property,
    d_current: P::Property,
}

impl<P: AnimatableProperty<Property: VectorSpace>> Dynamic<P> {
    pub fn new(target: P::Property, params: DynamicsParams) -> Self {
        Self {
            target,
            prev_target: target,
            params,

            current: target,
            d_current: <P::Property as VectorSpace>::ZERO,
        }
    }
}

impl<P: AnimatableProperty<Property: VectorSpace + Default>> Default for Dynamic<P> {
    fn default() -> Self {
        Self::new(Default::default(), Default::default())
    }
}

impl<P: AnimatableProperty<Property: VectorSpace>> Dynamic<P> {
    fn tick(&mut self, dt: f32, d_target: Option<P::Property>, k1: f32, k2: f32, k3: f32) {
        let d_target = d_target.unwrap_or_else(|| (self.target - self.prev_target) / dt);
        self.prev_target = self.target;

        self.current = self.current + self.d_current * dt;
        self.d_current = self.d_current
            + (self.target + d_target * k3 - self.current - self.d_current * k1) / k2 * dt;
    }

    fn tick_simple(&mut self, dt: f32, d_target: Option<P::Property>) {
        let DynamicsParams { k1, k2, k3, .. } = self.params;
        self.tick(dt, d_target, k1, k2, k3);
    }

    fn tick_stable(&mut self, dt: f32, d_target: Option<P::Property>) {
        let DynamicsParams { k1, k2, k3, .. } = self.params;
        let k2_stable = k2.max(k1 * dt).max((dt.squared() + k1 * dt) / 2.0);
        self.tick(dt, d_target, k1, k2_stable, k3);
    }

    fn tick_pole_matching(&mut self, dt: f32, d_target: Option<P::Property>) {
        let DynamicsParams {
            k1,
            k2,
            k3,
            w,
            z,
            d,
        } = self.params;

        let (k1_stable, k2_stable) = if w * dt < z {
            (k1, k2.max(dt * k1).max((dt.squared() + k1 * dt) / 2.0))
        } else {
            //use pole matching when the system is very fast
            let t1 = ops::exp(-z * w * dt);
            let alpha = 2.0
                * t1
                * if z <= 1.0 {
                    ops::cos(d * dt)
                } else {
                    ops::cosh(d * dt)
                };
            let beta = t1.squared();
            let t2 = (beta - alpha + 1.0).recip() * dt;
            (t2 * (1.0 - beta), t2 * dt)
        };

        self.tick(dt, d_target, k1_stable, k2_stable, k3);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DynamicsParams {
    k1: f32,
    k2: f32,
    k3: f32,
    w: f32,
    z: f32,
    d: f32,
}

impl Default for DynamicsParams {
    fn default() -> Self {
        Self::smooth_damp(1.0)
    }
}

impl DynamicsParams {
    pub fn new(frequency: f32, damping: f32, response: f32) -> Self {
        let w = 2.0 * PI * frequency;
        let z = damping;
        let d = w * (z * z - 1.0).abs().sqrt();

        let k1 = damping / (PI * frequency);
        let k2 = w.squared().recip();
        let k3 = damping * response / w;

        Self {
            k1,
            k2,
            k3,
            w,
            z,
            d,
        }
    }

    pub fn smooth_damp(frequency: f32) -> Self {
        Self::new(frequency, 1.0, 0.0)
    }
}

#[derive(Default)]
pub enum DynamicsTickMode {
    Simple,
    #[default]
    Stable,
    PoleMatching,
}

pub struct DynamicsPlugin<P: AnimatableProperty<Property: VectorSpace>> {
    tick_mode: DynamicsTickMode,
    _data: PhantomData<fn(P)>,
}

impl<P: AnimatableProperty<Property: VectorSpace>> DynamicsPlugin<P> {
    pub fn new(tick_mode: DynamicsTickMode) -> Self {
        Self {
            tick_mode,
            _data: PhantomData,
        }
    }
}

impl<P: AnimatableProperty<Property: VectorSpace>> Default for DynamicsPlugin<P> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<P: AnimatableProperty<Property: VectorSpace>> Plugin for DynamicsPlugin<P> {
    fn build(&self, app: &mut App) {
        match self.tick_mode {
            DynamicsTickMode::Simple => {
                app.add_systems(PreUpdate, tick_dynamics_simple::<P>);
            }
            DynamicsTickMode::Stable => {
                app.add_systems(PreUpdate, tick_dynamics_stable::<P>);
            }
            DynamicsTickMode::PoleMatching => {
                app.add_systems(PreUpdate, tick_dynamics_pole_matching::<P>);
            }
        }
    }
}

fn tick_dynamics_simple<P: AnimatableProperty<Property: VectorSpace>>(
    mut dynamics: Query<(&mut Dynamic<P>, &mut P::Component)>,
    time: Res<Time>,
) {
    if time.delta_seconds() <= 0.0 {
        return;
    }

    for (mut dynamic, mut component) in &mut dynamics {
        let Some(property) = P::get_mut(component.deref_mut()) else {
            continue;
        };

        dynamic.tick_simple(time.delta_seconds(), None);
        *property = dynamic.current;
    }
}

fn tick_dynamics_stable<P: AnimatableProperty<Property: VectorSpace>>(
    mut dynamics: Query<(&mut Dynamic<P>, &mut P::Component)>,
    time: Res<Time>,
) {
    if time.delta_seconds() <= 0.0 {
        return;
    }

    for (mut dynamic, mut component) in &mut dynamics {
        let Some(property) = P::get_mut(component.deref_mut()) else {
            continue;
        };

        dynamic.tick_stable(time.delta_seconds(), None);
        *property = dynamic.current;
    }
}

fn tick_dynamics_pole_matching<P: AnimatableProperty<Property: VectorSpace>>(
    mut dynamics: Query<(&mut Dynamic<P>, &mut P::Component)>,
    time: Res<Time>,
) {
    if time.delta_seconds() <= 0.0 {
        return;
    }

    for (mut dynamic, mut component) in &mut dynamics {
        let Some(property) = P::get_mut(component.deref_mut()) else {
            continue;
        };

        dynamic.tick_pole_matching(time.delta_seconds(), None);
        *property = dynamic.current;
    }
}
