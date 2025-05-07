use std::marker::PhantomData;

use crate::physics::{Direction, PhysicsSet};
use bevy::{
    app::{FixedMain, FixedUpdate, PostUpdate},
    gizmos::gizmos::Gizmos,
    math::Vec3Swizzles,
    prelude::{
        App, Bundle, Color, Component, Entity, GlobalTransform, IntoSystemConfigs,
        IntoSystemSetConfigs, Parent, Plugin, Query, ResMut, Schedule, SpatialBundle, Srgba,
        SystemSet, Transform, Vec2, Without,
    },
    transform::{
        systems::{propagate_transforms, sync_simple_transforms},
        TransformSystem::TransformPropagate,
    },
};

#[derive(Default)]
pub struct CollisionPlugin<T: Component + Clone>(PhantomData<T>);
impl<T> Plugin for CollisionPlugin<T>
where
    T: Component + Clone,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        Self::add_systems_to_fixed_update(app);
    }
}

impl<T> CollisionPlugin<T>
where
    T: Component + Clone,
{
    pub fn add_systems_to_schedule(schedule: &mut Schedule) {
        schedule
            .configure_sets(
                (
                    CollisionSets::TransformPropagateBefore,
                    CollisionSets::Produce,
                    CollisionSets::Consume,
                    CollisionSets::TransformPropagateAfter,
                )
                    .chain()
                    .in_set(Collision),
            )
            .add_systems(
                (
                    propagate_transforms,
                    sync_simple_transforms,
                    cleanup_buffers::<T>,
                )
                    .in_set(CollisionSets::TransformPropagateBefore),
            )
            .add_systems(
                (
                    check_ray_to_box_collisions::<T>,
                    check_box_to_box_collisions::<T>,
                )
                    .in_set(CollisionSets::Produce),
            );
    }

    /// this function shouold be considered to be on user side
    // pub fn add_systems_to_post_update(app: &mut App) {
    //     app.edit_schedule(PostUpdate, |schedule| {
    //         Self::add_systems_to_schedule(schedule);
    //         schedule.configure_sets(Collision.before(TransformPropagate));
    //     });
    // }

    pub fn add_systems_to_fixed_update(app: &mut App) {
        app.edit_schedule(FixedUpdate, |schedule| {
            Self::add_systems_to_schedule(schedule);
            schedule.configure_sets(Collision.after(PhysicsSet));

            schedule.add_systems(
                (propagate_transforms, sync_simple_transforms)
                    .after(CollisionSets::TransformPropagateAfter),
            );
        });
    }
}

impl<T> CollisionPlugin<T>
where
    T: Component + Clone,
{
    pub fn new() -> Self {
        Self(PhantomData::default())
    }
}

/// overarching set that contains the collision detection systems
#[derive(SystemSet, Eq, PartialEq, Hash, Debug, Clone)]
pub struct Collision;

#[derive(SystemSet, Eq, PartialEq, Hash, Debug, Clone)]
pub enum CollisionSets {
    /// propagates transforms after physics runs
    TransformPropagateBefore,
    /// systems that produce the collision events
    Produce,
    /// systems that consume the collision events,
    /// users systems that need to modify the transform
    /// should go here
    Consume,
    TransformPropagateAfter,
}

trait Shape {}

/// Transform for a Box is the center.
#[derive(Component, Default)]
pub struct Rect(pub Vec2);
impl Shape for Rect {} // TODO: make a derive macro for Shape

#[derive(Bundle, Default)]
pub struct RectBundle {
    rect: Rect,
    spatial_bundle: SpatialBundle,
}

impl RectBundle {
    pub fn new(size: Vec2) -> RectBundle {
        RectBundle {
            rect: Rect(size),
            spatial_bundle: SpatialBundle::default(),
        }
    }
}

/// `Transform` is the origin of the ray
#[derive(Component, Default)]
pub struct Ray(pub Vec2);
impl Shape for Ray {}

impl Ray {
    // algorithm adapted from here https://tavianator.com/2011/ray_box.html
    // may not handle collisions with corners correctly
    fn intersect_aabb(
        ray_origin: Vec2,
        ray: &Ray,
        box_center: Vec2,
        box_size: &Rect,
    ) -> Option<RayIntersection> {
        // calculate vectors to corners of box from ray origin
        let bottom_left = box_center - box_size.0 / 2.0 - ray_origin; // bottom left
        let top_right = box_center + box_size.0 / 2.0 - ray_origin; // top right

        // calculate intersections with extended lines of sides of box
        // t is position along ray
        let n_inv = ray.0.normalize().recip();
        let t_tr = top_right * n_inv;
        let t_bl = bottom_left * n_inv;

        let left = RayIntersection {
            normal: Direction::Left.as_vec2(),
            point: Vec2::default(),
            toi: t_bl.x,
            ray_direction: ray.0,
            ray_origin,
        };
        let right = RayIntersection {
            normal: Direction::Right.as_vec2(),
            point: Vec2::default(),
            toi: t_tr.x,
            ray_direction: ray.0,
            ray_origin,
        };
        let top = RayIntersection {
            normal: Direction::Up.as_vec2(),
            point: Vec2::default(),
            toi: t_tr.y,
            ray_direction: ray.0,
            ray_origin,
        };
        let bottom = RayIntersection {
            normal: Direction::Down.as_vec2(),
            point: Vec2::default(),
            toi: t_bl.y,
            ray_direction: ray.0,
            ray_origin,
        };

        let mut tmin = c_max(&c_min(&left, &right), &c_min(&top, &bottom));
        let mut tmax = c_min(&c_max(&left, &right), &c_max(&top, &bottom));

        if (tmax.toi < tmin.toi) // ray misses box completely
        || (tmin.toi < 0.0 && tmax.toi < 0.0) // points away from box
        || (tmin.toi < 0.0 && tmax.toi > ray.0.length()) // contained inside box
        || (tmin.toi > 0.0 && tmin.toi >= ray.0.length())
        // ends before box
        {
            None
        } else if tmin.toi >= 0.0 {
            // ray collides from outside box
            tmin.point = ray_origin + tmin.toi * ray.0.normalize();
            Some(tmin)
        } else {
            // ray collides from inside box
            tmax.point = ray_origin + tmax.toi * ray.0.normalize();
            Some(tmax)
        }
    }
}

#[derive(Bundle, Default)]
pub struct RayBundle {
    ray: Ray,
    spatial_bundle: SpatialBundle,
}

impl RayBundle {
    pub fn new(ray: Vec2, origin: Vec2) -> RayBundle {
        RayBundle {
            ray: Ray(ray),
            spatial_bundle: SpatialBundle {
                transform: Transform::from_translation(origin.extend(0.0)),
                ..SpatialBundle::default()
            },
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct AabbIntersection {
    /// penetration depth
    delta: Vec2,
    normal: Vec2,
    point: Vec2,
}

impl Rect {
    pub fn intersect_aabb(
        &self,
        self_pos: Vec2,
        box_pos: Vec2,
        box_size: Vec2,
    ) -> Option<AabbIntersection> {
        Self::inter_aabb(self_pos, self.0, box_pos, box_size)
    }

    /// check whether 2 aabb's intersect with the separating axis test
    /// `AabbInterssection::normal` normal on `a` aabb that collision happens.
    /// `AabbInterssection::point` point on `a` aabb that collision happens.
    /// `AabbInterssection::delta` add delta to b_pos to make colliders just touch.
    pub fn inter_aabb(
        a_pos: Vec2,
        a_size: Vec2,
        b_pos: Vec2,
        b_size: Vec2,
    ) -> Option<AabbIntersection> {
        let d = b_pos - a_pos;
        let p = (b_size + a_size) / 2. - d.abs();

        if p.x < 0. || p.y < 0. {
            return None;
        }

        if p.x < p.y {
            let sx = if d.x < 0. { -1. } else { 1. };

            Some(AabbIntersection {
                delta: Vec2::new(p.x * sx, 0.0),
                normal: Vec2::new(sx, 0.0),
                point: Vec2::new(a_pos.x + sx * a_size.x / 2.0, b_pos.y),
            })
        } else {
            let sy = if d.y < 0. { -1. } else { 1. };

            Some(AabbIntersection {
                delta: Vec2::new(0.0, p.y * sy),
                normal: Vec2::new(0.0, sy),
                point: Vec2::new(b_pos.x, a_pos.y + sy * a_size.y / 2.0),
            })
        }
    }

    /// delta is vector between current a_pos and next a_pos
    pub fn sweep_aabb(
        a_pos: Vec2,
        a_size: Vec2,
        b_pos: Vec2,
        b_size: Vec2,
        delta: Vec2,
    ) -> Option<Sweep> {
        if delta == Vec2::ZERO {
            let hit = Rect::inter_aabb(b_pos, b_size, a_pos, a_size);
            if let Some(hit) = hit {
                return Some(Sweep {
                    position: a_pos - hit.delta,
                    time: 0.,
                    normal: hit.normal,
                });
            } else {
                return None;
            }
        }

        let hit = Ray::intersect_aabb(a_pos, &Ray(delta), b_pos, &Rect(b_size + a_size));
        if let Some(hit) = hit {
            // let time = (hit.toi - std::f32::EPSILON).clamp(0., 1.); // toi is probably % of length of ray
            let position = a_pos + hit.toi * delta.normalize();
            // let d_norm = delta.normalize();
            // let hit_pos =
            //     (hit.point + d_norm * b_size / 2.).clamp(a_pos - a_size / 2., a_pos + a_size / 2.);

            Some(Sweep {
                position,
                time: hit.toi,
                normal: hit.normal,
            })
        } else {
            None
        }
    }
}

/// result for sweep aabb test
#[derive(PartialEq, Debug)]
pub struct Sweep {
    // position of `a` aabb for sweep test that keeps object outside of collider
    pub position: Vec2,
    // distance along cast ray for collision
    pub time: f32,
    // normal of surface that is collided
    pub normal: Vec2,
    // hit: Option<AabbIntersection>,
}

// TODO: figure out if this compiles to something branchless
fn c_min(a: &RayIntersection, b: &RayIntersection) -> RayIntersection {
    if a.toi < b.toi || (!a.toi.is_nan() && b.toi.is_nan()) {
        a.clone()
    } else {
        b.clone()
    }
}

fn c_max(a: &RayIntersection, b: &RayIntersection) -> RayIntersection {
    if a.toi > b.toi || (!a.toi.is_nan() && b.toi.is_nan()) {
        a.clone()
    } else {
        b.clone()
    }
}

#[derive(Clone, Debug)]
pub struct RayIntersection {
    /// distance until time of impact
    pub toi: f32,
    /// point of intersection
    pub point: Vec2,
    /// normal at point of intersection
    pub normal: Vec2,
    pub ray_origin: Vec2,
    pub ray_direction: Vec2,
}

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PositionDelta {
    pub origin: Vec2,
    pub ray: Vec2,
}

pub struct CollisionEvent<T> {
    // entity that was collided with
    pub entity: Entity,
    // type T that was stored on entity that was collided with
    pub user_type: T,
    pub data: CollisionData,
}

/// the enum is the type of collider that detected the event
pub enum CollisionData {
    Ray(RayIntersection),
    Aabb(Sweep),
}

impl CollisionData {
    pub fn normal(&self) -> Vec2 {
        match self {
            CollisionData::Ray(ref data) => data.normal,
            CollisionData::Aabb(ref data) => data.normal,
        }
    }
}

#[derive(Component)]
pub struct CollisionEvents<T> {
    pub buffer: Vec<CollisionEvent<T>>,
}

impl<T> CollisionEvents<T> {
    pub fn new() -> CollisionEvents<T> {
        CollisionEvents { buffer: Vec::new() }
    }
}

pub fn check_ray_to_box_collisions<T>(
    rays: Query<(&Ray, &GlobalTransform, &Parent), Without<Rect>>,
    rects: Query<(&Rect, &GlobalTransform, &Parent), Without<Ray>>,
    mut collision_takers: Query<&mut CollisionEvents<T>>,
    user_types: Query<&T>,
) where
    T: Component + Clone,
{
    // TODO: need to apply the rotation from the `GlobalTransform` to the ray too. can probably just apply the full affine transformation?
    rays.iter().for_each(|(ray, ray_origin, ray_owner)| {
        rects.iter().for_each(|(rect, rect_center, rect_owner)| {
            if let Ok(mut collision_events) = collision_takers.get_mut(ray_owner.get()) {
                let collision = Ray::intersect_aabb(
                    ray_origin.translation().xy(),
                    ray,
                    rect_center.translation().xy(),
                    rect,
                );
                if let Some(collision) = collision {
                    collision_events.buffer.push(CollisionEvent {
                        entity: rect_owner.get(),
                        user_type: user_types.get(rect_owner.get()).unwrap().clone(),
                        data: CollisionData::Ray(collision),
                    });
                }
            }
        });
    });
}

// todo: should only check for rects that are interactable? i.e. don't check ground-ground interactions somehow
// maybe just need collision layers
pub fn check_box_to_box_collisions<T>(
    rects: Query<(&Rect, &GlobalTransform, &Parent)>,
    user_types: Query<&T>,
    mut collision_takers: Query<(&mut CollisionEvents<T>, Option<&PositionDelta>)>,
) where
    T: Component + Clone,
{
    for [(r1, t1, p1), (r2, t2, p2)] in rects.iter_combinations() {
        if let Ok((mut collision_events, d)) = collision_takers.get_mut(p1.get()) {
            let PositionDelta { origin, ray } = d.copied().unwrap_or(PositionDelta {
                origin: t1.translation().truncate(),
                ray: Vec2::ZERO,
            });
            let collision = Rect::sweep_aabb(origin, r1.0, t2.translation().truncate(), r2.0, ray);
            if let Some(collision) = collision {
                collision_events.buffer.push(CollisionEvent {
                    entity: p2.get(),
                    user_type: user_types.get(p2.get()).unwrap().clone(),
                    data: CollisionData::Aabb(collision),
                });
            }
        }

        // TODO: pull the logic out into another function and just swap the inputs
        if let Ok((mut collision_events, d)) = collision_takers.get_mut(p2.get()) {
            let PositionDelta { origin, ray } = d.copied().unwrap_or(PositionDelta {
                origin: t2.translation().truncate(),
                ray: Vec2::ZERO,
            });
            let collision = Rect::sweep_aabb(origin, r2.0, t1.translation().truncate(), r1.0, ray);
            if let Some(collision) = collision {
                collision_events.buffer.push(CollisionEvent {
                    entity: p1.get(),
                    user_type: user_types.get(p1.get()).unwrap().clone(),
                    data: CollisionData::Aabb(collision),
                });
            }
        }
    }
}

fn cleanup_buffers<T>(mut buffers: Query<&mut CollisionEvents<T>>)
where
    T: Component + Clone,
{
    for mut events in &mut buffers {
        events.buffer.clear();
    }
}

pub struct CollisionDebugPlugin;
impl Plugin for CollisionDebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PostUpdate, draw_collision_shapes.after(Collision));
    }
}

fn draw_collision_shapes(
    mut gizmos: Gizmos,
    rays: Query<(&Ray, &GlobalTransform)>,
    rects: Query<(&Rect, &GlobalTransform)>,
) {
    for (r, t) in &rays {
        gizmos.line_2d(
            t.translation().truncate(),
            t.translation().truncate() + r.0,
            Srgba::RED,
        );
    }

    for (size, t) in &rects {
        gizmos.rect_2d(t.translation().truncate(), size.0, Srgba::RED);
    }
}

#[cfg(test)]
mod tests {
    // test for `Rect::intersect_aabb`
    mod intersect_aabb {
        use bevy::prelude::Vec2;

        use crate::collisions::{AabbIntersection, Rect};

        #[test]
        fn detects_collision() {
            let collisions = [
                ("right", [4., 0.], ([1., 0.], [1., 0.], [3., 0.])),
                ("left", [-4., 0.], ([-1., 0.], [-1., 0.], [-3., 0.])),
                ("top", [0., 4.], ([0., 1.], [0., 1.], [0., 3.])),
                ("bottom", [0., -4.], ([0., -1.], [0., -1.], [0., -3.])),
                ("touch right", [5., 0.], ([0., 0.], [1., 0.], [3., 0.])),
                ("touch left", [-5., 0.], ([0., 0.], [-1., 0.], [-3., 0.])),
                ("touch top", [0., 5.], ([0., 0.], [0., 1.], [0., 3.])),
                ("touch bottom", [0., -5.], ([0., 0.], [0., -1.], [0., -3.])),
                ("inside middle", [0., 0.], ([0., 5.], [0., 1.], [0., 3.])), // pushes out of top
                ("inside right", [0.1, 0.], ([4.9, 0.], [1., 0.], [3., 0.])),
                (
                    "inside left",
                    [-0.1, 0.],
                    ([-4.9, 0.], [-1., 0.], [-3., 0.]),
                ),
                ("inside top", [0., 0.1], ([0., 4.9], [0., 1.], [0., 3.])),
                (
                    "inside bottom",
                    [0., -0.1],
                    ([0., -4.9], [0., -1.], [0., -3.]),
                ),
            ];
            for col in collisions {
                let result = Rect::inter_aabb(
                    Vec2::new(0., 0.),
                    Vec2::new(6., 6.),
                    Vec2::from_array(col.1),
                    Vec2::new(4., 4.),
                );
                let expected_result = AabbIntersection {
                    delta: Vec2::from_array(col.2 .0),
                    normal: Vec2::from_array(col.2 .1),
                    point: Vec2::from_array(col.2 .2),
                };
                assert_eq!(
                    result.unwrap(),
                    expected_result,
                    "{} collision failed",
                    col.0
                );
            }
        }

        #[test]
        fn does_not_detect_collsions() {
            let not_collisions = [("right", [6., 0.]), ("top", [0., 6.])];
            for col in not_collisions {
                let result = Rect::inter_aabb(
                    Vec2::new(0., 0.),
                    Vec2::new(6., 6.),
                    Vec2::from_array(col.1),
                    Vec2::new(4., 4.),
                );
                assert!(result.is_none(), "{} collided unexectedly", col.0);
            }
        }
    }

    mod sweep_aabb {
        use bevy::prelude::Vec2;

        use crate::collisions::{Rect, Sweep};

        #[test]
        fn detects_collision() {
            let collisions = [
                ("left", ([10., 0.], [-10., 0.]), ([5., 0.], 5., [1., 0.])),
                ("right", ([-10., 0.], [10., 0.]), ([-5., 0.], 5., [-1., 0.])),
                ("top", ([0., 10.], [0., -10.]), ([0., 5.], 5., [0., 1.])),
                (
                    "bottom",                   // label for assert
                    ([0., -10.], [0., 10.]),    // (a_pos, delta)
                    ([0., -5.], 5., [0., -1.]), // expected_result (position, time, normal)
                ),
                ("no move", ([0., 3.], [0., 0.]), ([0., 5.], 0., [1., 0.])),
            ];
            for col in collisions {
                let result = Rect::sweep_aabb(
                    Vec2::from_array(col.1 .0),
                    Vec2::new(4., 4.),
                    Vec2::new(0., 0.),
                    Vec2::new(6., 6.),
                    Vec2::from_array(col.1 .1),
                );
                let expected_result = Sweep {
                    position: Vec2::from_array(col.2 .0),
                    time: col.2 .1,
                    normal: Vec2::from_array(col.2 .2),
                };
                assert_eq!(
                    result.unwrap(),
                    expected_result,
                    "{} collision failed",
                    col.0
                );
            }
        }

        #[test]
        fn does_not_detect_collision() {
            let collisions = [("miss", ([10., 10.], [-20., 0.]))];
            for col in collisions {
                let result = Rect::sweep_aabb(
                    Vec2::from_array(col.1 .0),
                    Vec2::new(4., 4.),
                    Vec2::new(0., 0.),
                    Vec2::new(6., 6.),
                    Vec2::from_array(col.1 .1),
                );
                assert!(result.is_none(), "{} collision detected", col.0);
            }
        }

        #[test]
        fn weird_case_causing_nans() {
            // the y values are the same, but not moving on y axis
            let result = Rect::sweep_aabb(
                Vec2::new(288., 372.),
                Vec2::new(24., 24.),
                Vec2::new(588., 420.),
                Vec2::new(72., 72.),
                Vec2::new(-0.05, 0.0),
            );
            assert!(result.is_none());
        }
    }
}
