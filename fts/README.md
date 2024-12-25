# Full-Text Search Service for Auditors

A Rust-based full-text search service for blockchain security auditors, using Tantivy for search indexing, MongoDB for data storage, and Sled for metadata persistence.

## Features

- Full-text search across multiple fields (first_name, last_name, about, company, tags)
- Custom filters:
  - Name-based (`name:"John Smith"`)
  - Company-based (`company:NewWorldAudits`)
  - Availability-based (`free_from:<2024-01-01`)
  - Tag-based (`#rust #smart_contract`)
  - Price range-based (`price_range:100..500`)
  - Rating range-based (`rating:3..5`)
- Sorting by:
  - Relevance (default)
  - Price (ascending/descending)
  - Rating (ascending/descending)
- Incremental updates with MongoDB synchronization
- Automatic cleanup of removed documents

## Requirements

- Rust 1.70 or later
- MongoDB 4.4 or later
- At least 1GB of free disk space for indexes

## Configuration

The service can be configured using environment variables:

- `MONGODB_URI` - MongoDB connection URI (default: "mongodb://localhost:27017")
- `DB_NAME` - MongoDB database name (default: "auditors")
- `HOST` - HTTP server host (default: "127.0.0.1")
- `PORT` - HTTP server port (default: 8080)

## API Endpoints

### GET /search

Search for auditors with various filters and sorting options.

Query parameters:
- `text` - Full-text search query
- `name` - Filter by auditor name
- `company` - Filter by company name
- `free_from` - Filter by availability date
- `tags` - Filter by tags (comma-separated)
- `price_range.from` - Minimum price
- `price_range.to` - Maximum price
- `rating.from` - Minimum rating
- `rating.to` - Maximum rating
- `sort` - Sorting option (relevance, price_asc, price_desc, rating_asc, rating_desc)
- `page` - Page number (default: 1)
- `per_page` - Results per page (default: 10)

Example:
```
GET /search?text=smart contract&tags=rust,solidity&price_range.from=100&price_range.to=500&sort=rating_desc
```

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