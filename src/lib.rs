#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use bevy::{
    prelude::{Entity, Vec2},
    reflect::Reflect,
    utils::{HashMap, HashSet},
};

/// Axis aligned bounding box
#[derive(Reflect, Debug, Default, Clone, Copy, PartialEq)]
pub struct Aabb {
    /// Lower left corner
    pub min: Vec2,
    /// Upper right corner
    pub max: Vec2,
}

type Key = (i32, i32);

fn key_from_point(point: Vec2) -> Key {
    (point.x as i32, point.y as i32)
}

/// A spatial container that allows querying for entities that share one or more grid cell
#[derive(Default, Reflect, Debug, Clone)]
pub struct SparseGrid2d {
    map: HashMap<Key, Vec<Entity>>,
}

// TODO: make const generic when stable
const TILE_SIZE: f32 = 1.5;

impl SparseGrid2d {
    /// Insert an entity in the given Aabb coordinates
    pub fn insert_aabb(&mut self, aabb: Aabb, entity: Entity) {
        for key in KeyIter::new(aabb) {
            self.map.entry(key).or_default().push(entity);
        }
    }

    /// Insert an entity at the given point coordinate
    pub fn insert_point(&mut self, point: Vec2, entity: Entity) {
        let key = key_from_point(point);
        self.map.entry(key).or_default().push(entity);
    }

    /// Get an iterator with the entities in the grid cells covered by the given Aabb
    ///
    /// may contain duplicates if some entities are in more than one grid cell
    #[inline]
    pub fn aabb_iter(&'_ self, aabb: Aabb) -> impl Iterator<Item = Entity> + '_ {
        KeyIter::new(aabb)
            .filter_map(|key| self.map.get(&key))
            .flatten()
            .copied()
    }

    /// Get an iterator with the entities in the grid cells at the given point
    #[inline]
    pub fn point_iter(&'_ self, point: Vec2) -> impl Iterator<Item = Entity> + '_ {
        let key = key_from_point(point);

        std::iter::once(key)
            .filter_map(|key| self.map.get(&key))
            .flatten()
            .copied()
    }

    /// Creates a hash set with all the entities in the grid cells covered by the given Aabb
    #[inline]
    pub fn query_aabb(&self, aabb: Aabb) -> HashSet<Entity> {
        self.aabb_iter(aabb).collect()
    }

    /// Remove all entities from the map
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

struct KeyIter {
    width: i32,
    start: Key,
    current: i32,
    count: i32,
}

impl KeyIter {
    fn new(Aabb { min, max }: Aabb) -> Self {
        // convert to key space
        let s = TILE_SIZE;
        let min = ((min.x / s).floor() as i32, (min.y / s).floor() as i32);
        let max = ((max.x / s).ceil() as i32, (max.y / s).ceil() as i32);
        let width = max.0 - min.0;
        let height = max.1 - min.1;
        let count = width * height;
        Self {
            start: min,
            current: -1,
            width,
            count,
        }
    }
}

impl Iterator for KeyIter {
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        self.current += 1;

        if self.current < self.count {
            Some((
                self.start.0 + self.current.rem_euclid(self.width),
                self.start.1 + self.current / self.width,
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::vec2;
    use bevy::utils::HashSet;

    use super::*;

    #[test]
    fn keys_single() {
        let keys: Vec<Key> = KeyIter::new(Aabb {
            min: vec2(0.001, 0.001),
            max: vec2(0.001, 0.001),
        })
        .collect();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], (0, 0));
    }

    #[test]
    fn keys_four_around_origin() {
        let keys: Vec<Key> = KeyIter::new(Aabb {
            min: vec2(-0.001, -0.001),
            max: vec2(0.001, 0.001),
        })
        .collect();
        assert!(keys.contains(&(0, 0)));
        assert!(keys.contains(&(0, -1)));
        assert!(keys.contains(&(-1, 0)));
        assert!(keys.contains(&(-1, -1)));
        assert_eq!(keys.len(), 4);
    }

    #[test]
    fn matches() {
        let entity = Entity::from_raw(123);
        let mut db = SparseGrid2d::default();
        db.insert_aabb(
            Aabb {
                min: vec2(-0.001, -0.001),
                max: vec2(0.001, 0.001),
            },
            entity,
        );

        let matches: Vec<Entity> = db
            .aabb_iter(Aabb {
                min: vec2(0.001, 0.001),
                max: vec2(0.001, 0.001),
            })
            .collect();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], entity);
    }

    #[test]
    fn key_negative() {
        let h = TILE_SIZE / 2.0;
        let keys: Vec<Key> = KeyIter::new(Aabb {
            min: vec2(-h, -h),
            max: vec2(-h, -h),
        })
        .collect();
        assert!(keys.contains(&(-1, -1)));
        assert_eq!(keys.len(), 1);
    }
    #[test]
    fn query_points() {
        let mut db = SparseGrid2d::default();
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        db.insert_point(vec2(0.5, 0.5), e1);
        db.insert_point(vec2(0.499, 0.501), e2);

        let matches: HashSet<_> = db.point_iter(vec2(0.499, 0.501)).collect();
        assert!(matches.contains(&e1));
        assert!(matches.contains(&e2));
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn matches_complex() {
        let h = TILE_SIZE / 2.0;
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let e3 = Entity::from_raw(3);
        let mut db = SparseGrid2d::default();
        db.insert_aabb(
            Aabb {
                min: vec2(-h, -h),
                max: vec2(h, h),
            },
            e1,
        );
        db.insert_aabb(
            Aabb {
                min: vec2(h, h),
                max: vec2(h, h),
            },
            e2,
        );
        db.insert_aabb(
            Aabb {
                min: vec2(-h, -h),
                max: vec2(-h, -h),
            },
            e3,
        );

        let matches: Vec<Entity> = db
            .aabb_iter(Aabb {
                min: vec2(-h, -h),
                max: vec2(h, h),
            })
            .collect();
        // assert_eq!(matches.len(), 3);
        assert!(matches.contains(&e1));
        assert!(matches.contains(&e2));
        assert!(matches.contains(&e3));

        let matches = db.query_aabb(Aabb {
            min: vec2(-0.001, -0.001),
            max: vec2(-0.001, -0.001),
        });
        assert_eq!(matches.len(), 2);
        assert!(matches.contains(&e1));
        assert!(matches.contains(&e3));

        let matches: Vec<Entity> = db
            .aabb_iter(Aabb {
                min: vec2(-0.001, -0.001),
                max: vec2(-0.001, -0.001),
            })
            .collect();
        assert_eq!(matches[0], e1);
    }
}
