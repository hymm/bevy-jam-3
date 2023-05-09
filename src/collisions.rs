use std::marker::PhantomData;

use crate::physics::Direction;
use bevy::{
    math::Vec3Swizzles,
    prelude::{
        Bundle, Color, Component, CoreSet, Entity, GlobalTransform, IntoSystemConfig,
        IntoSystemConfigs, IntoSystemSetConfigs, Parent, Plugin, Query, ResMut, SpatialBundle,
        SystemSet, Transform, Vec2, Vec3, Without,
    },
    transform::{
        systems::{propagate_transforms, sync_simple_transforms},
        TransformSystem::TransformPropagate,
    },
};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin, DebugShapes};

#[derive(Default)]
pub struct CollisionPlugin<T: Component + Clone>(PhantomData<T>);
impl<T> Plugin for CollisionPlugin<T>
where
    T: Component + Clone,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        // TODO: need to repropagate transforms after
        app.configure_sets(
            (
                CollisionSets::Produce,
                CollisionSets::Consume,
                CollisionSets::Repropagate,
            )
                .chain()
                .in_base_set(CoreSet::PostUpdate)
                .after(TransformPropagate),
        )
        .add_system(cleanup_buffers::<T>.before(CollisionSets::Produce))
        .add_systems(
            (
                check_ray_to_box_collisions::<T>,
                check_box_to_box_collisions::<T>,
            )
                .in_set(CollisionSets::Produce),
        )
        .add_systems(
            (propagate_transforms, sync_simple_transforms).in_set(CollisionSets::Repropagate),
        );
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

#[derive(SystemSet, Eq, PartialEq, Hash, Debug, Clone)]
pub enum CollisionSets {
    Produce,
    Consume,
    Repropagate,
}

trait Shape {}

/// Transform for a Box is the center.
#[derive(Component, Default)]
pub struct Rect(Vec2);
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
pub struct Ray(Vec2);
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

#[derive(Clone)]
pub enum RectSide {
    Top,
    Bottom,
    Left,
    Right,
    // one rect fully contained inside other
    Inside,
}

#[derive(PartialEq, Debug)]
pub struct AabbIntersection {
    /// penetration depth
    delta: Vec2,
    normal: Vec2,
    point: Vec2,
}

impl Rect {
    pub fn insersect_aabb(
        &self,
        self_pos: Vec2,
        box_pos: Vec2,
        box_size: Vec2,
    ) -> Option<AabbIntersection> {
        Self::intersect_aabb(self_pos, self.0, box_pos, box_size)
    }

    /// check whether 2 aabb's intersect with the separating axis test
    /// `AabbInterssection::normal` normal on `a` aabb that collision happens.
    /// `AabbInterssection::point` point on `a` aabb that collision happens.
    /// `AabbInterssection::delta` add delta to b_pos to make colliders just touch.
    pub fn intersect_aabb(
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
}

pub struct Sweep {
    position: Vec2,
    hit: Option<AabbIntersection>,
    time: f32,
}

/// delta is vector between current a_pos and next a_pos
fn sweepAABB(a_pos: Vec2, a_size: Vec2, b_pos: Vec2, b_size: Vec2, delta: Ray) -> Sweep {
    if delta.0 == Vec2::ZERO {
        let hit = Rect::intersect_aabb(a_pos, a_size, b_pos, b_size);
        let time = if hit.is_none() { 1. } else { 0. };
        return Sweep {
            position: b_pos,
            hit,
            time,
        };
    }

    let hit = Ray::intersect_aabb(a_pos, &delta, b_pos, &Rect(b_size));
    if let Some(hit) = hit {
        // let time = (hit.toi - std::f32::EPSILON).clamp(0., 1.); // toi is probably % of length of ray
        let position = b_pos + hit.toi;
        let d_norm = delta.0.normalize();
        let hit_pos =
            (hit.point + d_norm * b_size / 2.).clamp(a_pos - a_size / 2., a_pos + a_size / 2.);

        Sweep {
            position,
            time: hit.toi,
            hit: Some(AabbIntersection {
                delta: delta.0,
                normal: hit.normal,
                point: hit_pos,
            }),
        }
    } else {
        let hit_pos = b_pos + delta.0;
        let time = 1.;
        Sweep {
            position: hit_pos,
            time,
            hit: None,
        }
    }
}

fn c_min(a: &RayIntersection, b: &RayIntersection) -> RayIntersection {
    if a.toi < b.toi {
        a.clone()
    } else {
        b.clone()
    }
}

fn c_max(a: &RayIntersection, b: &RayIntersection) -> RayIntersection {
    if a.toi > b.toi {
        a.clone()
    } else {
        b.clone()
    }
}

#[derive(Clone)]
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

pub struct CollisionEvent<T> {
    pub entity: Entity,
    pub user_type: T,
    pub data: CollisionData,
}

/// the enum is the type of collider that detected the event
pub enum CollisionData {
    Ray(RayIntersection),
    Aabb(AabbIntersection),
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
    for (ray, ray_origin, ray_owner) in &rays {
        for (rect, rect_center, rect_owner) in &rects {
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
        }
    }
}

pub fn check_box_to_box_collisions<T>(
    rects: Query<(&Rect, &GlobalTransform, &Parent)>,
    user_types: Query<&T>,
    mut collision_takers: Query<&mut CollisionEvents<T>>,
) where
    T: Component + Clone,
{
    for [(r1, t1, p1), (r2, t2, p2)] in rects.iter_combinations() {
        if let Ok(mut collision_events) = collision_takers.get_mut(p1.get()) {
            let collision = Rect::intersect_aabb(
                t1.translation().truncate(),
                r1.0,
                t2.translation().truncate(),
                r2.0,
            );
            if let Some(collision) = collision {
                collision_events.buffer.push(CollisionEvent {
                    entity: p2.get(),
                    user_type: user_types.get(p2.get()).unwrap().clone(),
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
        app.add_plugin(DebugLinesPlugin::default()).add_system(
            draw_collision_shapes
                .in_base_set(CoreSet::PostUpdate)
                .after(CollisionSets::Repropagate),
        );
    }
}

fn draw_collision_shapes(
    mut lines: ResMut<DebugLines>,
    mut shapes: ResMut<DebugShapes>,
    rays: Query<(&Ray, &GlobalTransform)>,
    rects: Query<(&Rect, &GlobalTransform)>,
) {
    for (r, t) in &rays {
        lines.line_colored(
            t.translation(),
            t.translation() + r.0.extend(0.0),
            0.0,
            Color::RED,
        );
    }

    for (size, t) in &rects {
        shapes
            .rect()
            .size(size.0)
            .position(t.translation())
            .color(Color::RED);
    }
}

#[cfg(test)]
mod tests {
    // test for `Rect::intersect_aabb`
    mod intersect_aabb {
        use bevy::prelude::Vec2;

        use crate::collisions::{AabbIntersection, Rect};

        #[test]
        fn detects_collisions() {
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
                let result = Rect::intersect_aabb(
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
                let result = Rect::intersect_aabb(
                    Vec2::new(0., 0.),
                    Vec2::new(6., 6.),
                    Vec2::from_array(col.1),
                    Vec2::new(4., 4.),
                );
                assert!(result.is_none(), "{} collided unexectedly", col.0);
            }
        }
    }
}
