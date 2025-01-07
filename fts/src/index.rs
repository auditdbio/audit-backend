use std::path::Path;
use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use common::entities::{
    auditor::Auditor,
    contacts::Contacts,
    audit_request::PriceRange,
};
use tantivy::{
    schema::{Schema, STORED, TEXT, FAST, IndexRecordOption},
    Index, IndexReader, IndexWriter, TantivyDocument, Term,
    query::{QueryParser, Query, BooleanQuery, Occur, TermQuery, AllQuery},
    collector::TopDocs,
};
use tokio::sync::Mutex;
use futures::stream::BoxStream;

pub const MAX_LIMIT: usize = 100;

#[derive(Debug, Clone)]
pub struct PriceRangeFilter {
    pub from: Option<i64>,
    pub to: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct RatingFilter {
    pub from: Option<f32>,
    pub to: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct SearchFilter {
    pub query: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: Option<String>,
    pub company: Option<String>,
    pub tags: Option<Vec<String>>,
    pub price: Option<i64>,
    pub price_range: Option<PriceRangeFilter>,
    pub rating: Option<RatingFilter>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

pub struct AuditorIndex {
    index: Index,
    reader: IndexReader,
    writer: Arc<Mutex<IndexWriter>>,
    #[allow(dead_code)] // Schema is needed for index creation
    schema: Schema,
    // Store field references for quick access
    user_id: tantivy::schema::Field,
    avatar: tantivy::schema::Field,
    first_name: tantivy::schema::Field,
    last_name: tantivy::schema::Field,
    about: tantivy::schema::Field,
    company: tantivy::schema::Field,
    free_at: tantivy::schema::Field,
    tags: tantivy::schema::Field,
    contacts: tantivy::schema::Field,
    price_range: tantivy::schema::Field,
    last_modified: tantivy::schema::Field,
    created_at: tantivy::schema::Field,
    link_id: tantivy::schema::Field,
    rating: tantivy::schema::Field,
    full_text: tantivy::schema::Field,
}

#[async_trait]
pub trait AuditorSearch {
    async fn search(&self, filter: SearchFilter) -> Result<Vec<Auditor<String>>>;
    async fn add_or_update(&self, auditors: Vec<Auditor<String>>) -> Result<()>;
    async fn delete(&self, user_ids: Vec<String>) -> Result<()>;
    async fn stream(&self) -> Result<BoxStream<'static, Result<Auditor<String>>>>;
}

impl AuditorIndex {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut schema_builder = Schema::builder();

        // Define schema fields
        let user_id = schema_builder.add_text_field("user_id", TEXT | STORED);
        let avatar = schema_builder.add_text_field("avatar", STORED);
        let first_name = schema_builder.add_text_field("first_name", TEXT | STORED | FAST);
        let last_name = schema_builder.add_text_field("last_name", TEXT | STORED | FAST);
        let about = schema_builder.add_text_field("about", TEXT | STORED);
        let company = schema_builder.add_text_field("company", TEXT | STORED | FAST);
        let free_at = schema_builder.add_text_field("free_at", STORED);
        let tags = schema_builder.add_text_field("tags", TEXT | STORED | FAST);
        let contacts = schema_builder.add_text_field("contacts", STORED);
        let price_range = schema_builder.add_text_field("price_range", STORED);
        let last_modified = schema_builder.add_text_field("last_modified", STORED);
        let created_at = schema_builder.add_text_field("created_at", STORED);
        let link_id = schema_builder.add_text_field("link_id", STORED);
        let rating = schema_builder.add_text_field("rating", STORED);
        let full_text = schema_builder.add_text_field("full_text", TEXT);

        let schema = schema_builder.build();
        let index = Index::create_in_dir(path, schema.clone())?;
        let reader = index.reader()?;
        let writer = Arc::new(Mutex::new(index.writer(50_000_000)?)); // 50MB buffer

        Ok(Self {
            index,
            reader,
            writer,
            schema,
            user_id,
            avatar,
            first_name,
            last_name,
            about,
            company,
            free_at,
            tags,
            contacts,
            price_range,
            last_modified,
            created_at,
            link_id,
            rating,
            full_text,
        })
    }

    fn create_document(&self, auditor: &Auditor<String>) -> TantivyDocument {
        let mut doc = TantivyDocument::new();

        let company_lower = auditor.company.to_lowercase();
        println!("Indexing company: {} -> {}", auditor.company, company_lower);

        doc.add_text(self.user_id, &auditor.user_id);
        doc.add_text(self.avatar, &auditor.avatar);
        doc.add_text(self.first_name, &auditor.first_name.to_lowercase());
        doc.add_text(self.last_name, &auditor.last_name.to_lowercase());
        doc.add_text(self.about, &auditor.about);
        doc.add_text(self.company, &company_lower);
        doc.add_text(self.free_at, &auditor.free_at);
        doc.add_text(self.tags, &auditor.tags.iter().map(|t| t.to_lowercase()).collect::<Vec<_>>().join(" "));
        doc.add_text(self.contacts, &serde_json::to_string(&auditor.contacts).unwrap());
        doc.add_text(self.price_range, &serde_json::to_string(&auditor.price_range).unwrap());
        doc.add_text(self.last_modified, &auditor.last_modified.to_string());
        if let Some(created_at) = auditor.created_at {
            doc.add_text(self.created_at, &created_at.to_string());
        }
        if let Some(link_id) = &auditor.link_id {
            doc.add_text(self.link_id, link_id);
        }
        if let Some(rating) = auditor.rating {
            doc.add_text(self.rating, &rating.to_string());
        }

        // Create full text field for searching
        let full_text = format!(
            "{} {} {} {} {}",
            auditor.first_name.to_lowercase(),
            auditor.last_name.to_lowercase(),
            auditor.about.to_lowercase(),
            company_lower,
            auditor.tags.iter().map(|t| t.to_lowercase()).collect::<Vec<_>>().join(" ")
        );
        doc.add_text(self.full_text, &full_text);

        doc
    }

    fn document_to_auditor(&self, doc: TantivyDocument) -> Result<Auditor<String>> {
        let get_field = |field: tantivy::schema::Field| -> String {
            doc.get_first(field)
                .and_then(|f| serde_json::to_value(f).ok())
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_default()
        };

        let contacts: Contacts = serde_json::from_str(&get_field(self.contacts))?;
        let price_range: PriceRange = serde_json::from_str(&get_field(self.price_range))?;
        let rating = get_field(self.rating).parse::<f32>().ok();
        let created_at = get_field(self.created_at).parse::<i64>().ok();
        let last_modified = get_field(self.last_modified).parse::<i64>().unwrap_or_default();

        Ok(Auditor {
            user_id: get_field(self.user_id),
            avatar: get_field(self.avatar),
            first_name: get_field(self.first_name),
            last_name: get_field(self.last_name),
            about: get_field(self.about),
            company: get_field(self.company),
            free_at: get_field(self.free_at),
            tags: get_field(self.tags).split_whitespace().map(String::from).collect(),
            contacts,
            price_range,
            last_modified,
            created_at,
            link_id: Some(get_field(self.link_id)),
            rating,
        })
    }
}

#[async_trait]
impl AuditorSearch for AuditorIndex {
    async fn search(&self, filter: SearchFilter) -> Result<Vec<Auditor<String>>> {
        let searcher = self.reader.searcher();
        let mut query_terms: Vec<(Occur, Box<dyn Query>)> = Vec::new();

        // Full text search
        if let Some(query) = filter.query {
            let mut query_parser = QueryParser::for_index(&self.index, vec![self.full_text]);
            query_parser.set_conjunction_by_default(); // Make all terms required by default
            query_terms.push((Occur::Must, Box::new(query_parser.parse_query(&query)?)));
        }

        // First name exact match
        if let Some(first_name) = filter.first_name {
            println!("Searching for first name: {}", first_name);
            query_terms.push((
                Occur::Must,
                Box::new(TermQuery::new(
                    Term::from_field_text(self.first_name, &first_name.to_lowercase()),
                    IndexRecordOption::Basic,
                )),
            ));
        }

        // Last name exact match
        if let Some(last_name) = filter.last_name {
            println!("Searching for last name: {}", last_name);
            query_terms.push((
                Occur::Must,
                Box::new(TermQuery::new(
                    Term::from_field_text(self.last_name, &last_name.to_lowercase()),
                    IndexRecordOption::Basic,
                )),
            ));
        }

        // Full name exact match
        if let Some(full_name) = filter.full_name {
            println!("Searching for full name: {}", full_name);
            let parts: Vec<&str> = full_name.split_whitespace().collect();
            if parts.len() == 2 {
                query_terms.push((
                    Occur::Must,
                    Box::new(TermQuery::new(
                        Term::from_field_text(self.first_name, &parts[0].to_lowercase()),
                        IndexRecordOption::Basic,
                    )),
                ));
                query_terms.push((
                    Occur::Must,
                    Box::new(TermQuery::new(
                        Term::from_field_text(self.last_name, &parts[1].to_lowercase()),
                        IndexRecordOption::Basic,
                    )),
                ));
            }
        }

        // Company exact match
        if let Some(company) = filter.company {
            let company_lower = company.to_lowercase();
            println!("Searching for company: {} -> {}", company, company_lower);
            let mut company_parser = QueryParser::for_index(&self.index, vec![self.company]);
            company_parser.set_conjunction_by_default();
            query_terms.push((
                Occur::Must,
                Box::new(company_parser.parse_query(&format!("\"{}\"", company_lower))?),
            ));
        }

        // Tags filter
        if let Some(tags) = filter.tags {
            println!("Searching for tags: {:?}", tags);
            for tag in tags {
                query_terms.push((
                    Occur::Must,
                    Box::new(TermQuery::new(
                        Term::from_field_text(self.tags, &tag.to_lowercase()),
                        IndexRecordOption::Basic,
                    )),
                ));
            }
        }

        // Create final query
        let query = if query_terms.is_empty() {
            Box::new(AllQuery) as Box<dyn Query>
        } else {
            Box::new(BooleanQuery::new(query_terms))
        };

        // Execute search
        let limit = filter.limit.unwrap_or(MAX_LIMIT);
        let offset = filter.offset.unwrap_or(0);
        let total_limit = offset + limit;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(total_limit))?;
        println!("Found {} documents", top_docs.len());

        // Post-process results for price range and rating filters
        let mut results = Vec::new();
        for (_score, doc_address) in top_docs.iter().skip(offset) {
            let doc = searcher.doc::<TantivyDocument>(*doc_address)?;
            let auditor = self.document_to_auditor(doc)?;
            println!("Processing document: {} {} (company: {}, price: {}-{})", 
                auditor.first_name, 
                auditor.last_name, 
                auditor.company,
                auditor.price_range.from,
                auditor.price_range.to
            );

            // Apply price range filter
            if let Some(price_filter) = &filter.price_range {
                println!("Checking price range filter: {:?} against auditor's range: {}-{}", 
                    price_filter, auditor.price_range.from, auditor.price_range.to);

                let price_range = &auditor.price_range;
                let filter_from = price_filter.from.unwrap_or(i64::MIN);
                let filter_to = price_filter.to.unwrap_or(i64::MAX);

                // Check if auditor's range is fully contained within filter range
                if price_range.from < filter_from || price_range.to > filter_to {
                    println!("Skipping: auditor's range {}-{} is not fully contained within filter range {}-{}", 
                        price_range.from, price_range.to, filter_from, filter_to);
                    continue;
                }
            }

            // Apply price filter (single value)
            if let Some(price) = filter.price {
                println!("Checking price {} against auditor's range: {}-{}", 
                    price, auditor.price_range.from, auditor.price_range.to);

                // Check if price is within auditor's range (inclusive from, exclusive to)
                if price < auditor.price_range.from || price >= auditor.price_range.to {
                    println!("Skipping: price {} is not within auditor's range {}-{}", 
                        price, auditor.price_range.from, auditor.price_range.to);
                    continue;
                }
            }

            // Apply rating filter
            if let Some(rating_filter) = &filter.rating {
                if let Some(rating) = auditor.rating {
                    if let Some(from) = rating_filter.from {
                        if rating < from {
                            println!("Skipping due to rating (from)");
                            continue;
                        }
                    }
                    if let Some(to) = rating_filter.to {
                        if rating > to {
                            println!("Skipping due to rating (to)");
                            continue;
                        }
                    }
                } else {
                    println!("Skipping due to missing rating");
                    continue;
                }
            }

            results.push(auditor);
            if results.len() >= limit {
                break;
            }
        }

        println!("Returning {} results", results.len());
        Ok(results)
    }

    async fn add_or_update(&self, auditors: Vec<Auditor<String>>) -> Result<()> {
        let mut writer = self.writer.lock().await;
        for auditor in auditors {
            // Delete existing document if it exists
            writer.delete_term(Term::from_field_text(self.user_id, &auditor.user_id));
            
            // Add new document
            let doc = self.create_document(&auditor);
            writer.add_document(doc)?;
        }

        // Commit changes and reload reader
        writer.commit()?;
        self.reader.reload()?;
        Ok(())
    }

    async fn delete(&self, user_ids: Vec<String>) -> Result<()> {
        let mut writer = self.writer.lock().await;
        for user_id in user_ids {
            writer.delete_term(Term::from_field_text(self.user_id, &user_id));
        }
        writer.commit()?;
        self.reader.reload()?;
        Ok(())
    }

    async fn stream(&self) -> Result<BoxStream<'static, Result<Auditor<String>>>> {
        let searcher = self.reader.searcher();
        let query = AllQuery;
        let total_docs = searcher.num_docs() as usize;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(total_docs))?;

        // Clone the searcher and index fields we need
        let searcher = searcher.clone();
        let user_id = self.user_id;
        let avatar = self.avatar;
        let first_name = self.first_name;
        let last_name = self.last_name;
        let about = self.about;
        let company = self.company;
        let free_at = self.free_at;
        let tags = self.tags;
        let contacts = self.contacts;
        let price_range = self.price_range;
        let last_modified = self.last_modified;
        let created_at = self.created_at;
        let link_id = self.link_id;
        let rating = self.rating;

        let stream = futures::stream::iter(top_docs.into_iter().map(move |(_score, doc_address)| {
            let doc = searcher.doc::<TantivyDocument>(doc_address)?;
            
            let get_field = |field: tantivy::schema::Field| -> String {
                doc.get_first(field)
                    .and_then(|f| serde_json::to_value(f).ok())
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_default()
            };

            let contacts_str = get_field(contacts);
            let price_range_str = get_field(price_range);
            let contacts: Contacts = serde_json::from_str(&contacts_str)?;
            let price_range: PriceRange = serde_json::from_str(&price_range_str)?;
            let rating_val = get_field(rating).parse::<f32>().ok();
            let created_at_val = get_field(created_at).parse::<i64>().ok();
            let last_modified_val = get_field(last_modified).parse::<i64>().unwrap_or_default();

            Ok(Auditor {
                user_id: get_field(user_id),
                avatar: get_field(avatar),
                first_name: get_field(first_name),
                last_name: get_field(last_name),
                about: get_field(about),
                company: get_field(company),
                free_at: get_field(free_at),
                tags: get_field(tags).split_whitespace().map(String::from).collect(),
                contacts,
                price_range,
                last_modified: last_modified_val,
                created_at: created_at_val,
                link_id: Some(get_field(link_id)),
                rating: rating_val,
            })
        }));

        Ok(Box::pin(stream))
    }
} 

#[cfg(test)]
mod tests {
    use super::*;

    use common::entities::{
        auditor::Auditor,
        contacts::Contacts,
        audit_request::PriceRange,
    };
    use tempfile::TempDir;
    use futures::StreamExt;

    fn create_sample_auditor(
        user_id: &str,
        first_name: &str,
        last_name: &str,
        company: &str,
        about: &str,
        tags: Vec<String>,
        rating: Option<f32>,
        price_range: PriceRange,
    ) -> Auditor<String> {
        Auditor {
            user_id: user_id.to_string(),
            avatar: "avatar.jpg".to_string(),
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            about: about.to_string(),
            company: company.to_string(),
            free_at: "2024-01-01".to_string(),
            tags,
            contacts: Contacts {
                email: Some("test@example.com".to_string()),
                telegram: Some("@test".to_string()),
                public_contacts: true,
            },
            price_range,
            last_modified: 1704614400,
            created_at: Some(1704614400),
            link_id: Some("test-link".to_string()),
            rating,
        }
    }

    #[tokio::test]
    async fn test_add_and_search_basic() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let index = AuditorIndex::new(temp_dir.path())?;

        let auditor = create_sample_auditor(
            "user1",
            "John",
            "Doe",
            "Test Company",
            "About me text",
            vec!["rust".to_string(), "blockchain".to_string()],
            Some(4.5),
            PriceRange { from: 100, to: 200 },
        );

        // Add auditor to index
        index.add_or_update(vec![auditor.clone()]).await?;

        // Search with empty filter
        let results = index
            .search(SearchFilter {
                query: None,
                first_name: None,
                last_name: None,
                full_name: None,
                company: None,
                tags: None,
                price: None,
                price_range: None,
                rating: None,
                offset: None,
                limit: None,
            })
            .await?;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].user_id, auditor.user_id);

        Ok(())
    }

    #[tokio::test]
    async fn test_search_with_filters() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let index = AuditorIndex::new(temp_dir.path())?;

        let auditors = vec![
            create_sample_auditor(
                "user1",
                "John",
                "Doe",
                "Tech Corp",
                "Rust developer",
                vec!["rust".to_string(), "blockchain".to_string()],
                Some(4.5),
                PriceRange { from: 100, to: 200 },
            ),
            create_sample_auditor(
                "user2",
                "Jane",
                "Smith",
                "Dev Inc",
                "Security expert",
                vec!["security".to_string(), "blockchain".to_string()],
                Some(4.8),
                PriceRange { from: 150, to: 300 },
            ),
            create_sample_auditor(
                "user3",
                "John",
                "Smith",
                "Tech Corp",
                "Web developer",
                vec!["web".to_string(), "javascript".to_string()],
                Some(4.2),
                PriceRange { from: 80, to: 150 },
            ),
        ];

        // Add auditors to index
        index.add_or_update(auditors.clone()).await?;

        // Test full text search
        let results = index
            .search(SearchFilter {
                query: Some("rust blockchain".to_string()),
                first_name: None,
                last_name: None,
                full_name: None,
                company: None,
                tags: None,
                price: None,
                price_range: None,
                rating: None,
                offset: None,
                limit: None,
            })
            .await?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].user_id, "user1");

        // Test first name filter
        let results = index
            .search(SearchFilter {
                query: None,
                first_name: Some("John".to_string()),
                last_name: None,
                full_name: None,
                company: None,
                tags: None,
                price: None,
                price_range: None,
                rating: None,
                offset: None,
                limit: None,
            })
            .await?;
        assert_eq!(results.len(), 2);

        // Test company filter
        let results = index
            .search(SearchFilter {
                query: None,
                first_name: None,
                last_name: None,
                full_name: None,
                company: Some("Tech Corp".to_string()),
                tags: None,
                price: None,
                price_range: None,
                rating: None,
                offset: None,
                limit: None,
            })
            .await?;
        assert_eq!(results.len(), 2);

        // Test tags filter
        let results = index
            .search(SearchFilter {
                query: None,
                first_name: None,
                last_name: None,
                full_name: None,
                company: None,
                tags: Some(vec!["blockchain".to_string()]),
                price: None,
                price_range: None,
                rating: None,
                offset: None,
                limit: None,
            })
            .await?;
        assert_eq!(results.len(), 2);

        // Test price filter (exact value)
        let results = index
            .search(SearchFilter {
                query: None,
                first_name: None,
                last_name: None,
                full_name: None,
                company: None,
                tags: None,
                price: Some(150),
                price_range: None,
                rating: None,
                offset: None,
                limit: None,
            })
            .await?;
        assert_eq!(results.len(), 2); // Should match user1 (100-200) and user2 (150-300)

        let results = index
            .search(SearchFilter {
                query: None,
                first_name: None,
                last_name: None,
                full_name: None,
                company: None,
                tags: None,
                price: Some(250),
                price_range: None,
                rating: None,
                offset: None,
                limit: None,
            })
            .await?;
        assert_eq!(results.len(), 1); // Should only match user2 (150-300)

        // Test price range filter (range containment)
        let results = index
            .search(SearchFilter {
                query: None,
                first_name: None,
                last_name: None,
                full_name: None,
                company: None,
                tags: None,
                price: None,
                price_range: Some(PriceRangeFilter {
                    from: Some(100),
                    to: Some(200),
                }),
                rating: None,
                offset: None,
                limit: None,
            })
            .await?;
        assert_eq!(results.len(), 1); // Should only match user1 (100-200) as it's fully contained

        // Test rating filter
        let results = index
            .search(SearchFilter {
                query: None,
                first_name: None,
                last_name: None,
                full_name: None,
                company: None,
                tags: None,
                price: None,
                price_range: None,
                rating: Some(RatingFilter {
                    from: Some(4.5),
                    to: None,
                }),
                offset: None,
                limit: None,
            })
            .await?;
        assert_eq!(results.len(), 2);

        // Test combined filters
        let results = index
            .search(SearchFilter {
                query: None,
                first_name: Some("John".to_string()),
                last_name: None,
                full_name: None,
                company: Some("Tech Corp".to_string()),
                tags: Some(vec!["rust".to_string()]),
                price: None,
                price_range: None,
                rating: None,
                offset: None,
                limit: None,
            })
            .await?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].user_id, "user1");

        Ok(())
    }

    #[tokio::test]
    async fn test_delete() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let index = AuditorIndex::new(temp_dir.path())?;

        let auditor = create_sample_auditor(
            "user1",
            "John",
            "Doe",
            "Test Company",
            "About me text",
            vec!["rust".to_string(), "blockchain".to_string()],
            Some(4.5),
            PriceRange { from: 100, to: 200 },
        );

        // Add auditor to index
        index.add_or_update(vec![auditor.clone()]).await?;

        // Delete auditor
        index.delete(vec![auditor.user_id.clone()]).await?;

        // Search with empty filter
        let results = index
            .search(SearchFilter {
                query: None,
                first_name: None,
                last_name: None,
                full_name: None,
                company: None,
                tags: None,
                price: None,
                price_range: None,
                rating: None,
                offset: None,
                limit: None,
            })
            .await?;

        assert_eq!(results.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_stream() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let index = AuditorIndex::new(temp_dir.path())?;

        let auditors = vec![
            create_sample_auditor(
                "user1",
                "John",
                "Doe",
                "Tech Corp",
                "Rust developer",
                vec!["rust".to_string(), "blockchain".to_string()],
                Some(4.5),
                PriceRange { from: 100, to: 200 },
            ),
            create_sample_auditor(
                "user2",
                "Jane",
                "Smith",
                "Dev Inc",
                "Security expert",
                vec!["security".to_string(), "blockchain".to_string()],
                Some(4.8),
                PriceRange { from: 150, to: 300 },
            ),
        ];

        // Add auditors to index
        index.add_or_update(auditors.clone()).await?;

        // Stream all auditors
        let mut stream = index.stream().await?;
        let mut count = 0;
        while let Some(result) = stream.next().await {
            let auditor = result?;
            assert!(auditors.iter().any(|a| a.user_id == auditor.user_id));
            count += 1;
        }
        assert_eq!(count, auditors.len());

        Ok(())
    } 
}