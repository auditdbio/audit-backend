**Product Requirements Document (PRD)**  
**Project:** FTS for a Social Network of Blockchain Security Auditors  
**Technology Stack:** Rust, Tantivy (v0.22), MongoDB, Sled  

---

## 1. Introduction
We are building a full-text search (FTS) solution for a social network of blockchain security auditors. The core data resides in a MongoDB collection called `auditors`. The FTS engine uses Tantivy (v0.22) and persists its index on disk. Additional metadata (e.g., `last_modified` offsets) is tracked via the Sled key-value store.  

The system must support custom filters (similar to GitHub’s advanced search syntax) alongside full-text queries. We also need incremental updates—only newly added or updated auditors should be ingested on a scheduled basis, and documents that no longer exist in MongoDB should be removed from the search index.

---

## 2. Goals and Objectives
1. **Full-Text Search**: Provide robust, high-speed search across multiple textual fields (`first_name`, `last_name`, `about`, `company`, `tags`).  
2. **Custom Filters**:  
   - Name-based (`name:"John Smith"`).  
   - Company-based (`company:NewWorldAudits`).  
   - Availability-based (`free_from:<2024-01-01`).  
   - Tag-based (`#rust #smart_contract`).  
   - Price range-based.  
   - Rating range-based.  
3. **Sorting**:  
   - Relevance to search query.  
   - Price range (ascending/descending).  
   - Rating (ascending/descending).  
4. **Incremental Updates**: Leverage `last_modified` to fetch only newly created or updated documents from MongoDB, storing the highest known timestamp in Sled for scheduling.  
5. **Cleanup**: Periodically verify if indexed documents still exist in MongoDB; remove them from the index if not found.  

---

## 3. Data Model

### 3.1 MongoDB Collection: `auditors`
Each document in the `auditors` collection has the following structure:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auditor {
    pub id: ObjectId,
    pub user_id: ObjectId,
    pub avatar: String,
    pub first_name: String,    // Included in FTS & name filter
    pub last_name: String,     // Included in FTS & name filter
    pub about: String,         // Included in FTS
    pub company: String,       // Included in FTS & company filter
    pub free_at: String,       // Used for custom time interval filter
    pub tags: Vec<String>,     // Included in FTS & #tags filter
    pub contacts: Contacts,
    pub price_range: PriceRange,  // Used for custom filter
    pub last_modified: i64,    // Used for incremental updates
    pub created_at: Option<i64>,
    pub link_id: Option<String>,
    pub rating: Option<f32>,   // Used for custom filter
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contacts {
    pub email: Option<String>,
    pub telegram: Option<String>,
    pub public_contacts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRange {
    pub from: i64,
    pub to: i64,
}
```

### 3.2 Index Fields
Within Tantivy, we will store indexed and tokenized fields for full-text search, plus non-tokenized fields for filters/sorting. Key points:
- **FTS Fields**: `first_name`, `last_name`, `about`, `company`, `tags`
- **Filter-Only Fields**: `free_at`, `price_range`, `rating`, `id` (ObjectId, stored as string or Bytes), etc.  
- **Sort-Only Fields**: We can store `rating` and normalized price in numeric fields for sorting. Relevance is automatically handled by Tantivy for text fields.

---

## 4. Indexing Approach

1. **Tantivy Directory**: The index will be stored on disk using Tantivy’s default directory.  
2. **Incremental Sync**:  
   - Maintain a `last_synced_timestamp` in Sled.  
   - During each sync operation, query MongoDB for all `auditors` where `last_modified` > `last_synced_timestamp`.  
   - Update (add or replace) these documents in Tantivy.  
   - If new maximum `last_modified` is found, store it back in Sled.  
3. **Cleanup**:  
   - On a scheduled interval, iterate through all documents in the Tantivy index.  
   - Check if the corresponding `ObjectId` still exists in MongoDB.  
   - If not, remove the document from the index.  

---

## 5. Query Parsing

### 5.1 Custom GitHub-like Syntax
Users can enter queries such as:
```
name:"John Smith" free_from:<2024-05-01 #rust company:NewWorldAudits smart contract auditor
```

We parse the query into two components:
1. **FTS Query**: Terms that are not identified as filter keys (e.g., `smart contract auditor`).  
2. **Filter Criteria**:  
   - `name:"John Smith"` → Filters by `(first_name == "John" && last_name == "Smith")` or partial logic if user typed multiple words.  
   - `company:NewWorldAudits` → Filters by `company == "NewWorldAudits"`.  
   - `#rust #smart_contract` → Filters by `tags` containing both `rust` and `smart_contract`.  
   - `free_from:<2024-05-01` → Filters out auditors whose `free_at` is later than `2024-05-01`. (Conversely, `free_from:>=2024-05-01` can filter the other way.)  
   - `rating:3..5` → Filters by `3.0 <= rating <= 5.0`.  
   - `price_range:100..500` → Filters by `price_range.from >= 100 && price_range.to <= 500` (or handle partial intervals if only `from` or `to` is specified).  

**Notes**:  
- If no operator is specified for intervals, default to inclusive range.  
- If only a start value is given (e.g., `price_range:>=100`), treat it as `[100, ∞)`.  
- If only an end value is given (e.g., `rating:<=4`), treat it as `(-∞, 4]`.  

### 5.2 Filter Execution
- Create a Boolean filter that must match all user-specified conditions.  
- Combine that with the full-text query for relevant scoring.

---

## 6. Filtering Requirements

**1. Name-based filter**  
   - Syntax: `name:"John Smith"`  
   - We may parse the string inside quotes and match it against `first_name` and `last_name`.  

**2. Company-based filter**  
   - Syntax: `company:NewWorldAudits`  
   - Strict equality check with the `company` field.  

**3. Time-based filter (Availability)**  
   - Syntax: `free_from:<2024-05-01` or `free_from:>2024-05-01`.  
   - Compare to `free_at` (ISO date strings).  
   - `free_at` indicates when an auditor becomes free, so `free_from:<X` returns auditors who are free before X.  

**4. Tag-based filter**  
   - Syntax: `#rust #smart_contract #defi`  
   - All listed tags must be present in the `tags` array.  

**5. Price range filter**  
   - Syntax examples:  
     - `price_range:100..500`  
     - `price_range:>=100`  
     - `price_range:<=500`  
   - Compare against `price_range.from` and `price_range.to`.  

**6. Rating filter**  
   - Syntax examples:  
     - `rating:3..5`  
     - `rating:>=4`  
     - `rating:<=3.5`  
   - Compare numeric range with the `rating` field.  

---

## 7. Sorting Requirements
1. **Relevance** (default)  
2. **Price** (`price_range`)  
   - May decide to sort by `price_range.from`, or an average `(from + to)/2`.  
3. **Rating** (descending or ascending)  

Users specify sorting via a parameter (e.g., `sort:price asc`, `sort:rating desc`). If unspecified, default is relevance.

---

## 8. Data Synchronization and Background Tasks

### 8.1 Incremental Loading
- **Sled Key**: `last_synced_timestamp` (i64).  
- **Process**:  
  1. Read `last_synced_timestamp` from Sled.  
  2. Query MongoDB: `auditors` where `last_modified` > this timestamp.  
  3. Upsert documents into Tantivy.  
  4. Update Sled with the new max `last_modified`.  

### 8.2 Cleanup
- Scheduled job (e.g., every 24 hours):  
  1. Iterate through all documents in the Tantivy index.  
  2. For each `id`, check MongoDB if it exists.  
  3. If absent, remove from index.  

---

## 9. Storage Mechanisms

1. **Tantivy**:  
   - Stores the FTS index on disk (posting lists, term dictionary, doc store, etc.).  
   - Index schema includes all fields required for searching, filtering, and sorting.  

2. **Sled**:  
   - Key-value pairs for storing:  
     - `last_synced_timestamp` (the highest `last_modified` encountered).  
     - Possibly other metadata like checkpoint statuses, if needed.  

3. **MongoDB**:  
   - Source of truth for auditor data.  

---

## 10. Performance Considerations
- **Index Size**: Large sets of text in `about`, `tags`, etc., can lead to higher disk usage. Must monitor and potentially optimize tokenization.  
- **Update Frequency**: The `last_modified`-based approach keeps overhead low, loading only changed documents.  
- **Concurrent Queries**: Tantivy is designed to handle concurrent readers. Writes (index commits) must be carefully scheduled to avoid blocking.  

---

## 11. Timeline & Milestones
1. **Schema Definition**: Finalize Tantivy schema and Sled structure.  
2. **Indexing Prototype**: Implement an initial index load of existing `auditors`.  
3. **Query Parser**: Implement custom GitHub-like syntax parser.  
4. **Filtering & Sorting**: Integrate filter logic with Tantivy’s query handling.  
5. **Incremental Updates**: Add logic to sync based on `last_modified`.  
6. **Cleanup Process**: Implement background job for index pruning.  
7. **Testing & Optimization**: Load tests, query performance measurements, refine tokenization.  

---

## 12. Limitations & Next Steps
- **Partial Word Matching**: Depending on the chosen tokenizer, partial matches or wildcard queries may need special handling.  
- **Stemming & Language Support**: This PRD focuses on basic English tokenization. Additional languages might require specialized analyzers.  
- **Advanced Relevance Ranking**: Future improvements could involve dynamic scoring (e.g., weighting `first_name` and `last_name` differently).  
- **Security & Permissions**: Currently, all audits are publicly searchable. Future versions might require ACL or custom visibility rules.  

---

### Summary
This PRD outlines the requirements and design for a Rust-based FTS using Tantivy, backed by MongoDB and Sled for metadata. The system will enable robust, filtered searching of auditors with minimal overhead, providing advanced filters and incremental index updates.

### PS

All comments in code should be in English only.
Use actix-web = "4.9.0" and actix-cors = "0.6.4" for search api.

VERY IMPORTANT: 
when you consider the development is over, run `cargo check` to ensure that there are no errors.