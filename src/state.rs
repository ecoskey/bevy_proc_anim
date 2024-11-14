use crate::AnimValue;

use std::{f32::consts::PI, fmt::Debug, marker::PhantomData};

use bevy_app::App;
use bevy_app::{Plugin, PreUpdate};
use bevy_ecs::system::SystemId;
use bevy_ecs::{
    component::Component,
    entity::Entity,
    prelude::Local,
    query::{QueryState, With},
    schedule::{IntoSystemConfigs, SystemSet},
    system::{Query, Res},
    world::World,
};
use bevy_math::ops::{self, FloatPow};
use bevy_time::Time;

pub struct AnimHandle<T: AnimValue> {
    pub(crate) entity: Entity,
    _data: PhantomData<T>,
}

impl<T: AnimValue> Copy for AnimHandle<T> {}

impl<T: AnimValue> Clone for AnimHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: AnimValue> PartialEq for AnimHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.entity == other.entity
    }
}

impl<T: AnimValue> Eq for AnimHandle<T> {}

impl<T: AnimValue> AnimHandle<T> {
    pub(crate) fn new(entity: Entity) -> Self {
        Self {
            entity,
            _data: PhantomData,
        }
    }
}

#[derive(Component)]
pub(crate) struct DynamicsState<T: AnimValue> {
    pub target: T,
    pub params: DynamicsParams,

    prev_target: T,
    current: T,
    d_current: T,
}

impl<T: AnimValue> DynamicsState<T> {
    pub fn new(target: T, params: DynamicsParams) -> Self {
        Self {
            target,
            prev_target: target,
            params,

            current: target,
            d_current: T::ZERO,
        }
    }

    pub fn value(&self) -> T {
        self.current
    }
}

impl<T: AnimValue> Default for DynamicsState<T> {
    fn default() -> Self {
        Self::new(Default::default(), Default::default())
    }
}

impl<T: AnimValue> DynamicsState<T> {
    fn tick(&mut self, dt: f32, d_target: Option<T>, k1: f32, k2: f32, k3: f32) {
        let d_target = d_target.unwrap_or_else(|| (self.target - self.prev_target) / dt);
        self.prev_target = self.target;

        self.current = self.current + self.d_current * dt;
        self.d_current = self.d_current
            + (self.target + d_target * k3 - self.current - self.d_current * k1) / k2 * dt;
    }

    fn tick_simple(&mut self, dt: f32, d_target: Option<T>) {
        let DynamicsParams { k1, k2, k3, .. } = self.params;
        self.tick(dt, d_target, k1, k2, k3);
    }

    fn tick_stable(&mut self, dt: f32, d_target: Option<T>) {
        let DynamicsParams { k1, k2, k3, .. } = self.params;
        let k2_stable = k2.max(k1 * dt).max((dt.squared() + k1 * dt) / 2.0);
        self.tick(dt, d_target, k1, k2_stable, k3);
    }

    fn tick_pole_matching(&mut self, dt: f32, d_target: Option<T>) {
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

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TickMode {
    Simple,
    #[default]
    Stable,
    PoleMatching,
}

pub struct AnimValuePlugin<T: AnimValue>(PhantomData<fn(T)>);

impl<T: AnimValue> Default for AnimValuePlugin<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: AnimValue> Plugin for AnimValuePlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_sources::<T>.in_set(DynamicsSet::Read))
            .add_systems(
                PreUpdate,
                (
                    tick_dynamics_simple::<T>,
                    tick_dynamics_stable::<T>,
                    tick_dynamics_pole_matching::<T>,
                )
                    .in_set(DynamicsSet::Tick),
            );
    }
}

#[derive(Component)]
pub(crate) struct TickSimple;

#[derive(Component)]
pub(crate) struct TickStable;

#[derive(Component)]
pub(crate) struct TickPoleMatching;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, SystemSet)]
pub enum DynamicsSet {
    All,
    Read,
    Tick,
    Write,
}

pub(super) fn non_zero_delta(time: Res<Time>) -> bool {
    time.delta_secs() > 0.0
}

fn tick_dynamics_simple<T: AnimValue>(
    mut dynamics: Query<&mut DynamicsState<T>, With<TickSimple>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    dynamics
        .iter_mut()
        .for_each(|mut d| d.tick_simple(dt, None));
}

fn tick_dynamics_stable<T: AnimValue>(
    mut dynamics: Query<&mut DynamicsState<T>, With<TickStable>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    dynamics
        .iter_mut()
        .for_each(|mut d| d.tick_stable(dt, None));
}

fn tick_dynamics_pole_matching<T: AnimValue>(
    mut dynamics: Query<&mut DynamicsState<T>, With<TickPoleMatching>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    dynamics
        .iter_mut()
        .for_each(|mut d| d.tick_pole_matching(dt, None));
}

#[derive(Component)]
pub(crate) struct DynamicsSource<T: AnimValue>(pub SystemId<(), T>);

//TODO: GROSS
#[allow(clippy::type_complexity)]
fn update_sources<T: AnimValue>(
    world: &mut World,
    query: &mut QueryState<Entity, (With<DynamicsState<T>>, With<DynamicsSource<T>>)>,
    mut entities: Local<Vec<Entity>>,
) {
    entities.extend(query.iter(world));
    for entity in entities.drain(..) {
        let mut entity = world.entity_mut(entity);
        let callback_id = entity.get::<DynamicsSource<T>>().unwrap().0;
        let Ok(val) = entity.world_scope(|world| world.run_system(callback_id)) else {
            //TODO: warn
            continue;
        };
        entity.get_mut::<DynamicsState<T>>().unwrap().target = val;
    }
}
