Develop a full-text search engine module using `tantivy` 0.22.0 . Search should store the following objects:

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

Where first_name, last_name, about, company, tags should be in fulltext index (just sum it all into one string).

Also we should support the additinal filtering options (all of them should be optional):
0. full text search over the noted fields
1. `first_name` should be equals to the search string (case insensitive)
2. `last_name` should be equals to the search string (case insensitive)
2. `{first_name} {last_name}` should be equals to the search string (case insensitive)
3. `company` should be equals to the search string (case insensitive)
4. `tags` all given tags should be present in the `tags` array
5. `price_range` given integer should be inside the `price_range` range
6. `price_range` should be inside the given range (could have `from`, `to`, or both. If absent, the range is unlimited for that side)
7. `rating` should be inside the given range (could have `from`, `to`, or both. If absent, the range is unlimited for that side)

The search should be paginated with `offset` and `limit` parameters. These parameters should be optional. If not provided, the search should return up to`MAX_LIMIT` items.


The API should implement the async trait with the following methods:

1. Search (returns list of auditors)
2. Add or Update (add or update list of auditors in the index)
3. Delete (delete list of auditors by `user_id`s from the index)
4. Stream (return async iterator over all auditors in the index)


The implementation should be in `index` module of the `fts` crate. The object should be created with path of data directory, where to store the index.

Write unit tests for the `index` module. Generate the sample data and test the search functionality. Use temporary directory for the index. Implement and check tests one by one for better debug.

Run `cargo clippy` after each edit, espessialy edit of `Cargo.toml`, to prevent errors.





