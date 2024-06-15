# bevy_sparse_grid_2d

[![crates.io](https://img.shields.io/crates/v/bevy_sparse_grid_2d.svg)](https://crates.io/crates/bevy_sparse_grid_2d)
![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)

An opinionated 2D sparse grid made for use with Bevy. For storing and querying entities.

Personally, I'm using it for simple stupid collision broad phase in a couple of my projects.

## Usage

```rust
use bevy::{
    math::{bounding::Aabb2d, vec2},
    prelude::*,
    utils::HashSet,
};
use bevy_sparse_grid_2d::SparseGrid2d;
const TILE_SIZE: usize = 1; // how wide each cell is

let mut db = SparseGrid2d::<TILE_SIZE>::default();
let e1 = Entity::from_raw(1);
let e2 = Entity::from_raw(2);
db.insert_point(vec2(0.5, 0.5), e1);
db.insert_point(vec2(0.499, 0.501), e2);

// query a single point
let matches: HashSet<_> = db.point_iter(vec2(0.499, 0.501)).collect();
assert!(matches.contains(&e1));
assert!(matches.contains(&e2));
assert_eq!(matches.len(), 2);

// query an aabb
let aabb = Aabb2d::new(Vec2::new(0.001, 0.001), Vec2::new(0.001, 0.001));
let matches = db.query_aabb(aabb);
assert!(matches.contains(&e1));
assert!(matches.contains(&e2));
assert_eq!(matches.len(), 2);
```

See tests in lib.rs

## Bevy Version Support

The `main` branch targets the latest bevy release.

|bevy|bevy_sparse_grid_2d|
|----|-------------------|
|0.13|0.4, main          |
|0.12|0.3                |
|0.11|0.2                |
|0.10|0.1                |

## License

MIT or Apache-2.0

## Contributions

PRs welcome!