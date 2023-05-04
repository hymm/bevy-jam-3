use std::marker::PhantomData;

use crate::physics::Direction;
use bevy::{
    math::Vec3Swizzles,
    prelude::{
        Component, CoreSet, Entity, GlobalTransform, IntoSystemConfig, IntoSystemSetConfig, Parent,
        Plugin, Query, SystemSet, Vec2, Vec3, Without,
    },
};

#[derive(Default)]
pub struct CollisionPlugin<T: Component + CollisionType + Clone>(PhantomData<T>);
impl<T> Plugin for CollisionPlugin<T>
where
    T: Component + CollisionType + Clone,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        app.configure_set(CollisionEventChecking.in_base_set(CoreSet::PostUpdate))
            .add_system(check_ray_to_box_collisons::<T>.in_set(CollisionEventChecking))
            .add_system(check_box_to_box_collisons::<T>.in_set(CollisionEventChecking));
    }
}

impl<T> CollisionPlugin<T>
where
    T: Component + CollisionType + Clone,
{
    pub fn new() -> Self {
        Self(PhantomData::default())
    }
}

#[derive(SystemSet, Eq, PartialEq, Hash, Debug, Clone)]
pub struct CollisionEventChecking;

trait Shape {}

/// Transform for a Box is the center.
#[derive(Component)]
pub struct Rect(Vec2);
impl Shape for Rect {} // TODO: make a derive macro for Shape

/// `Transform` is the origin of the ray
#[derive(Component)]
pub struct Ray(Vec2);
impl Shape for Ray {}

#[derive(Clone)]
pub enum RectSide {
    Top,
    Bottom,
    Left,
    Right,
    // one rect fully contained inside other
    Inside,
}

pub struct RectCollisionEvent {
    pub side: RectSide,
    pub collision_type: Box<dyn CollisionType + Send + Sync>,
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
    /// distance until time of implact
    pub toi: f32,
    // /// point of intersection
    // pub point: Vec2,
    /// normal at point of intersection
    pub normal: Vec2,
}

pub trait CollisionType {}

pub struct RayCollisionEvent {
    /// id of entity being collided with
    pub entity: Entity,
    ///
    pub collision_type: Box<dyn CollisionType + Send + Sync>,
    ///
    pub intersection: RayIntersection,
}

/// the enum is the type of collider that detected the event
pub enum CollisionEvent {
    Ray(RayCollisionEvent),
    Rect(RectCollisionEvent),
}

#[derive(Component)]
pub struct CollisionEvents {
    pub buffer: Vec<CollisionEvent>,
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
    };
    let right = RayIntersection {
        normal: Direction::Right.as_vec2(),
        toi: t_tr.x,
    };
    let top = RayIntersection {
        normal: Direction::Up.as_vec2(),
        toi: t_tr.y,
    };
    let bottom = RayIntersection {
        normal: Direction::Down.as_vec2(),
        toi: t_bl.y,
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

pub fn check_ray_to_box_collisons<T>(
    rays: Query<(&Ray, &GlobalTransform, &Parent), Without<Rect>>,
    rects: Query<(&Rect, &GlobalTransform, &Parent, &T), Without<Ray>>,
    mut collision_takers: Query<&mut CollisionEvents>,
) where
    T: Component + CollisionType + Clone,
{
    for (ray, ray_origin, ray_owner) in &rays {
        for (rect, rect_center, rect_owner, t) in &rects {
            let collision = raycast_to_box(
                ray_origin.translation().xy(),
                ray,
                rect_center.translation().xy(),
                rect,
            );
            if let Some(collision) = collision {
                collision_takers
                    .get_mut(ray_owner.get())
                    .unwrap()
                    .buffer
                    .push(CollisionEvent::Ray(RayCollisionEvent {
                        entity: rect_owner.get(),
                        collision_type: Box::new(t.clone()),
                        intersection: collision,
                    }));
            }
        }
    }
}

pub fn check_box_to_box_collisons<T>(
    rects: Query<(&Rect, &GlobalTransform, &Parent, &T)>,
    mut collision_takers: Query<&mut CollisionEvents>,
) where
    T: Component + CollisionType + Clone,
{
    for [(r1, t1, p1, _ct1), (r2, t2, _p2, ct2)] in rects.iter_combinations() {
        let collision = collide_aabb(t1.translation(), r1.0, t2.translation(), r2.0);
        if let Some(collision) = collision {
            collision_takers
                .get_mut(p1.get())
                .unwrap()
                .buffer
                .push(CollisionEvent::Rect(RectCollisionEvent {
                    side: collision,
                    collision_type: Box::new(ct2.clone()),
                }));
        }
    }
}
