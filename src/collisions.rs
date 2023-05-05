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

pub fn collide_aabb(a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> Option<RectSide> {
    let a_min = a_pos.truncate() - a_size / 2.0;
    let a_max = a_pos.truncate() + a_size / 2.0;

    let b_min = b_pos.truncate() - b_size / 2.0;
    let b_max = b_pos.truncate() + b_size / 2.0;

    // check to see if the two rectangles are intersecting
    if a_min.x < b_max.x && a_max.x > b_min.x && a_min.y < b_max.y && a_max.y > b_min.y {
        // check to see if we hit on the left or right side
        let (x_collision, x_depth) = if a_min.x < b_min.x && a_max.x > b_min.x && a_max.x < b_max.x
        {
            (RectSide::Left, b_min.x - a_max.x)
        } else if a_min.x > b_min.x && a_min.x < b_max.x && a_max.x > b_max.x {
            (RectSide::Right, a_min.x - b_max.x)
        } else {
            (RectSide::Inside, -f32::INFINITY)
        };

        // check to see if we hit on the top or bottom side
        let (y_collision, y_depth) = if a_min.y < b_min.y && a_max.y > b_min.y && a_max.y < b_max.y
        {
            (RectSide::Bottom, b_min.y - a_max.y)
        } else if a_min.y > b_min.y && a_min.y < b_max.y && a_max.y > b_max.y {
            (RectSide::Top, a_min.y - b_max.y)
        } else {
            (RectSide::Inside, -f32::INFINITY)
        };

        // if we had an "x" and a "y" collision, pick the "primary" side using penetration depth
        if y_depth.abs() < x_depth.abs() {
            Some(y_collision)
        } else {
            Some(x_collision)
        }
    } else {
        None
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
    // /// point of intersection
    // pub point: Vec2,
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
    Rect(RectSide),
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

// algorithm adapted from here https://tavianator.com/2011/ray_box.html
// may not handle collisions with corners correctly
fn raycast_to_box(
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
        toi: t_bl.x,
        ray_direction: ray.0,
        ray_origin,
    };
    let right = RayIntersection {
        normal: Direction::Right.as_vec2(),
        toi: t_tr.x,
        ray_direction: ray.0,
        ray_origin,
    };
    let top = RayIntersection {
        normal: Direction::Up.as_vec2(),
        toi: t_tr.y,
        ray_direction: ray.0,
        ray_origin,
    };
    let bottom = RayIntersection {
        normal: Direction::Down.as_vec2(),
        toi: t_bl.y,
        ray_direction: ray.0,
        ray_origin,
    };

    let tmin = c_max(&c_min(&left, &right), &c_min(&top, &bottom));
    let tmax = c_min(&c_max(&left, &right), &c_max(&top, &bottom));

    if (tmax.toi < tmin.toi) // ray misses box completely
        || (tmin.toi < 0.0 && tmax.toi < 0.0) // points away from box
        || (tmin.toi < 0.0 && tmax.toi > ray.0.length()) // contained inside box
        || (tmin.toi > 0.0 && tmin.toi >= ray.0.length())
    // ends before box
    {
        None
    } else if tmin.toi >= 0.0 {
        // ray collides from outside box
        Some(tmin)
    } else {
        // ray collides from inside box
        Some(tmax)
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
                let collision = raycast_to_box(
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
            let collision = collide_aabb(t1.translation(), r1.0, t2.translation(), r2.0);
            if let Some(collision) = collision {
                collision_events.buffer.push(CollisionEvent {
                    entity: p2.get(),
                    user_type: user_types.get(p2.get()).unwrap().clone(),
                    data: CollisionData::Rect(collision),
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
