Implement the `database` module for the `fts` crate.

The database should work with `Auditor` structure in `auditors` collection of MongoDB.

```rust
///from common::entities::auditor::Auditor

pub struct Auditor<Id> {
    pub user_id: Id,
    pub avatar: String,
    pub first_name: String,
    pub last_name: String,
    pub about: String,
    pub company: String,
    pub free_at: String,
    pub tags: Vec<String>,
    pub contacts: Contacts,
    pub price_range: PriceRange,
    pub last_modified: i64,
    pub created_at: Option<i64>,
    pub link_id: Option<String>,
    pub rating: Option<f32>,
}
```

Async trait should support the following methods:

1. Check precense of the objects in `auditors` collection by Vec of `user_id` (MongoDB ObjectId or String serialized ObjectId), returns Vec<bool>
2. Get list of all objects with last_modified greater or equal to the given value


Async trait should be implemented for the object with MongoDB client inside.


Testing should be based on https://github.com/testcontainers/testcontainers-rs-modules-community


Write unit tests for the `database` module. Generate the sample data and test the search functionality. Use `testcontainers-modules` for temporary database. Use sample generated data for the tests. Implement and check tests one by one for better debug.

Run `cargo clippy` after each edit, espessialy edit of `Cargo.toml`, to prevent errors.
