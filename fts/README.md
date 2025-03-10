# Full-Text Search Service for Auditors

A Rust-based full-text search service for blockchain security auditors, using Tantivy for search indexing, MongoDB for data storage, and Sled for metadata persistence.

## Features

- Full-text search across multiple fields (first_name, last_name, about, company, tags)
- Custom filters:
  - Name-based (`name:"John Smith"`)
  - Company-based (`company:NewWorldAudits`)
  - Availability-based (`free_at:2024-01-01`)
  - Tag-based (`rust,smart_contract`)
  - Price range-based (`price_range:100..500`)
  - Rating range-based (`rating:3..5`)
- Sorting by:
  - Relevance (default)
  - Price (ascending/descending)
  - Rating (ascending/descending)
- Incremental updates with MongoDB synchronization
- Automatic synchronization
- Automatic cleanup of removed documents

## Requirements

- Rust 1.70 or later
- MongoDB 4.4 or later
- At least 1GB of free disk space for indexes

## Configuration

The service can be configured using environment variables:

- `MONGOURI` - MongoDB connection URI (default: "mongodb://localhost:27017")

## API Endpoints

### GET /search

Search for auditors with various filters and sorting options.

Query parameters:
- `q` - Composite search query (see detailed explanation below)
- `text` - Full-text search query
- `name` - Filter by name
- `company` - Filter by company name
- `free_at` - Filter by availability date
- `tags` - Filter by tags (comma-separated)
- `price_from` - Minimum price
- `price_to` - Maximum price
- `rating_from` - Minimum rating
- `rating_to` - Maximum rating
- `sort` - Sorting option (relevance, price_asc, price_desc, rating_asc, rating_desc)
- `page` - Page number (default: 1)
- `per_page` - Results per page (default: 10)
- `partial_match` - Enable partial match
- `kind` - Search kind. Required (comma-separated - auditor, badge, customer, project)

Example:
```
GET /search?text=smart contract&tags=rust,solidity&price_from=100&price_to=500&sort=rating_desc&kind=auditor
```

#### Composite Search Query (`q` parameter)

The `q` parameter allows you to combine multiple search criteria in a single parameter. It supports the following formats:

- Simple text search: `q=blockchain security`
- Field-specific filters:
  - `name:value` - Filter by name (If the name is separated by a space, use double quotes)
  - `company:value` - Filter by company
  - `tags:value1,value2` - Filter by tags (comma-separated)
  - `free_at:date` - Filter by availability date
  - `price:range` - Filter by price range
  - `rating:range` - Filter by rating range

For price and rating ranges, the following formats are supported:
- Range notation: `price:100..200` (from 100 to 200)
- Greater than: `price:>100` (more than 100)
- Less than: `price:<200` (less than 200)
- Single value:
  - For price: `price:100` (interpreted as price_to=100)
  - For rating: `rating:4` (interpreted as rating_from=4)

You can combine multiple criteria in a single query:

GET /search?q=smart contract+name:"john smith"+company:auditdb+tags:solidity,rust+price:<200+rating:10..100&kind=auditor

### POST /sync

Synchronize the search index with MongoDB. Use `force=true` query parameter to rebuild the index from scratch.

Example:
```
POST /sync?force=true
```

### POST /cleanup

Remove documents from the search index that no longer exist in MongoDB.

Example:
```
POST /cleanup
```

## Development

1. Clone the repository
2. Install dependencies:
   ```bash
   cargo build
   ```
3. Run the service:
   ```bash
   cargo run
   ```

## Testing

Run the test suite:
```bash
cargo test
```

## License

MIT 
