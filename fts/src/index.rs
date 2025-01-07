use std::path::Path;
use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use common::entities::{
    auditor::Auditor,
    contacts::Contacts,
    audit_request::PriceRange,
};
use log::{debug, info, warn};
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
pub struct DateRangeFilter {
    pub from: Option<String>,
    pub to: Option<String>,
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
    pub free_at_range: Option<DateRangeFilter>,
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
    first_name_lowercase: tantivy::schema::Field,
    last_name: tantivy::schema::Field,
    last_name_lowercase: tantivy::schema::Field,
    about: tantivy::schema::Field,
    company: tantivy::schema::Field,
    company_lowercase: tantivy::schema::Field,
    free_at: tantivy::schema::Field,
    tags: tantivy::schema::Field,
    tags_lowercase: tantivy::schema::Field,
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
        let first_name = schema_builder.add_text_field("first_name", STORED);
        let first_name_lowercase = schema_builder.add_text_field("first_name_lowercase", TEXT | FAST);
        let last_name = schema_builder.add_text_field("last_name", STORED);
        let last_name_lowercase = schema_builder.add_text_field("last_name_lowercase", TEXT | FAST);
        let about = schema_builder.add_text_field("about", TEXT | STORED);
        let company = schema_builder.add_text_field("company", STORED);
        let company_lowercase = schema_builder.add_text_field("company_lowercase", TEXT | FAST);
        let free_at = schema_builder.add_text_field("free_at", STORED);
        let tags = schema_builder.add_text_field("tags", STORED);
        let tags_lowercase = schema_builder.add_text_field("tags_lowercase", TEXT | FAST);
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
            first_name_lowercase,
            last_name,
            last_name_lowercase,
            about,
            company,
            company_lowercase,
            free_at,
            tags,
            tags_lowercase,
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
        debug!("Indexing company: {} -> {}", auditor.company, company_lower);

        doc.add_text(self.user_id, &auditor.user_id);
        doc.add_text(self.avatar, &auditor.avatar);
        doc.add_text(self.first_name, &auditor.first_name);
        doc.add_text(self.first_name_lowercase, &auditor.first_name.to_lowercase());
        doc.add_text(self.last_name, &auditor.last_name);
        doc.add_text(self.last_name_lowercase, &auditor.last_name.to_lowercase());
        doc.add_text(self.about, &auditor.about);
        doc.add_text(self.company, &auditor.company);
        doc.add_text(self.company_lowercase, &company_lower);
        doc.add_text(self.free_at, &auditor.free_at);
        let tags_joined = auditor.tags.join(" ");
        doc.add_text(self.tags, &tags_joined);
        doc.add_text(self.tags_lowercase, &tags_joined.to_lowercase());
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
        if let Some(query) = &filter.query {
            debug!("Performing full text search with query: {}", query);
            let mut query_parser = QueryParser::for_index(&self.index, vec![self.full_text]);
            query_parser.set_conjunction_by_default();
            query_terms.push((Occur::Must, Box::new(query_parser.parse_query(query)?)));
        }

        // First name exact match
        if let Some(first_name) = &filter.first_name {
            debug!("Searching for first name: {}", first_name);
            query_terms.push((
                Occur::Must,
                Box::new(TermQuery::new(
                    Term::from_field_text(self.first_name_lowercase, &first_name.to_lowercase()),
                    IndexRecordOption::Basic,
                )),
            ));
        }

        // Last name exact match
        if let Some(last_name) = &filter.last_name {
            debug!("Searching for last name: {}", last_name);
            query_terms.push((
                Occur::Must,
                Box::new(TermQuery::new(
                    Term::from_field_text(self.last_name_lowercase, &last_name.to_lowercase()),
                    IndexRecordOption::Basic,
                )),
            ));
        }

        // Full name exact match
        if let Some(full_name) = &filter.full_name {
            debug!("Searching for full name: {}", full_name);
            let parts: Vec<&str> = full_name.split_whitespace().collect();
            if parts.len() == 2 {
                query_terms.push((
                    Occur::Must,
                    Box::new(TermQuery::new(
                        Term::from_field_text(self.first_name_lowercase, &parts[0].to_lowercase()),
                        IndexRecordOption::Basic,
                    )),
                ));
                query_terms.push((
                    Occur::Must,
                    Box::new(TermQuery::new(
                        Term::from_field_text(self.last_name_lowercase, &parts[1].to_lowercase()),
                        IndexRecordOption::Basic,
                    )),
                ));
            } else {
                warn!("Invalid full name format: {}", full_name);
            }
        }

        // Company exact match
        if let Some(company) = &filter.company {
            let company_lower = company.to_lowercase();
            debug!("Searching for company: {} -> {}", company, company_lower);
            let mut company_parser = QueryParser::for_index(&self.index, vec![self.company_lowercase]);
            company_parser.set_conjunction_by_default();
            query_terms.push((
                Occur::Must,
                Box::new(company_parser.parse_query(&format!("\"{}\"", company_lower))?),
            ));
        }

        // Tags filter
        if let Some(tags) = &filter.tags {
            debug!("Searching for tags: {:?}", tags);
            for tag in tags {
                query_terms.push((
                    Occur::Must,
                    Box::new(TermQuery::new(
                        Term::from_field_text(self.tags_lowercase, &tag.to_lowercase()),
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
        debug!("Pagination: offset={}, limit={}, total_limit={}", offset, limit, total_limit);

        let top_docs = searcher.search(&query, &TopDocs::with_limit(total_limit))?;
        info!("Found {} documents before pagination", top_docs.len());

        // Post-process results for price range and rating filters
        let mut results = Vec::new();
        for (idx, (_score, doc_address)) in top_docs.iter().enumerate() {
            debug!("Processing document {} of {}", idx + 1, top_docs.len());
            let doc = searcher.doc::<TantivyDocument>(*doc_address)?;
            let auditor = self.document_to_auditor(doc)?;
            debug!("Document {}: {} {} (user_id: {})", 
                idx + 1, 
                auditor.first_name, 
                auditor.last_name,
                auditor.user_id
            );

            // Apply price range filter
            if let Some(price_filter) = &filter.price_range {
                debug!("Checking price range filter: {:?} against auditor's range: {}-{}", 
                    price_filter, auditor.price_range.from, auditor.price_range.to);

                let price_range = &auditor.price_range;
                let filter_from = price_filter.from.unwrap_or(i64::MIN);
                let filter_to = price_filter.to.unwrap_or(i64::MAX);

                // Check if auditor's range is fully contained within filter range
                if price_range.from < filter_from || price_range.to > filter_to {
                    debug!("Skipping: auditor's range {}-{} is not fully contained within filter range {}-{}", 
                        price_range.from, price_range.to, filter_from, filter_to);
                    continue;
                }
            }

            // Apply price filter (single value)
            if let Some(price) = filter.price {
                debug!("Checking price {} against auditor's range: {}-{}", 
                    price, auditor.price_range.from, auditor.price_range.to);

                // Check if price is within auditor's range (inclusive from, exclusive to)
                if price < auditor.price_range.from || price >= auditor.price_range.to {
                    debug!("Skipping: price {} is not within auditor's range {}-{}", 
                        price, auditor.price_range.from, auditor.price_range.to);
                    continue;
                }
            }

            // Apply rating filter
            if let Some(rating_filter) = &filter.rating {
                if let Some(rating) = auditor.rating {
                    if let Some(from) = rating_filter.from {
                        if rating < from {
                            debug!("Skipping due to rating below minimum ({})", from);
                            continue;
                        }
                    }
                    if let Some(to) = rating_filter.to {
                        if rating > to {
                            debug!("Skipping due to rating above maximum ({})", to);
                            continue;
                        }
                    }
                } else {
                    debug!("Skipping due to missing rating");
                    continue;
                }
            }

            // Apply free_at date range filter
            if let Some(date_filter) = &filter.free_at_range {
                debug!("Checking free_at date filter: {:?} against auditor's free_at: {}", 
                    date_filter, auditor.free_at);

                if let Some(from) = &date_filter.from {
                    if auditor.free_at < *from {
                        debug!("Skipping: auditor's free_at {} is before filter from date {}", 
                            auditor.free_at, from);
                        continue;
                    }
                }
                if let Some(to) = &date_filter.to {
                    if auditor.free_at > *to {
                        debug!("Skipping: auditor's free_at {} is after filter to date {}", 
                            auditor.free_at, to);
                        continue;
                    }
                }
            }

            results.push(auditor);
        }

        // Apply offset and limit after all filters
        let start = offset.min(results.len());
        let end = (offset + limit).min(results.len());
        debug!("Applying pagination: start={}, end={}, total_results={}", start, end, results.len());
        results = results.into_iter().skip(start).take(end - start).collect();

        info!("Returning {} results", results.len());
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

    #[ctor::ctor]
    fn setup() {
        env_logger::init();
    }

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
                free_at_range: None,
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
                free_at_range: None,
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
                free_at_range: None,
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
                free_at_range: None,
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
                free_at_range: None,
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
                free_at_range: None,
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
                free_at_range: None,
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
                free_at_range: None,
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
                free_at_range: None,
                offset: None,
                limit: None,
            })
            .await?;
        assert_eq!(results.len(), 2);

        // Test date range filter
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
                free_at_range: Some(DateRangeFilter {
                    from: Some("2024-01-01".to_string()),
                    to: Some("2024-12-31".to_string()),
                }),
                offset: None,
                limit: None,
            })
            .await?;
        assert_eq!(results.len(), 3); // All test auditors have free_at: "2024-01-01"

        // Test date range filter with no matches
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
                free_at_range: Some(DateRangeFilter {
                    from: Some("2025-01-01".to_string()),
                    to: None,
                }),
                offset: None,
                limit: None,
            })
            .await?;
        assert_eq!(results.len(), 0); // No auditors available after 2025-01-01

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
                free_at_range: Some(DateRangeFilter {
                    from: Some("2024-01-01".to_string()),
                    to: Some("2024-12-31".to_string()),
                }),
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
                free_at_range: None,
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

    #[tokio::test]
    async fn test_search_pagination() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let index = AuditorIndex::new(temp_dir.path())?;

        // Create 5 auditors with different names for easy identification
        let auditors = vec![
            create_sample_auditor(
                "user1",
                "Alice",
                "Smith",
                "Tech Corp",
                "Developer",
                vec!["rust".to_string()],
                Some(4.5),
                PriceRange { from: 100, to: 200 },
            ),
            create_sample_auditor(
                "user2",
                "Bob",
                "Johnson",
                "Tech Corp",
                "Developer",
                vec!["rust".to_string()],
                Some(4.5),
                PriceRange { from: 100, to: 200 },
            ),
            create_sample_auditor(
                "user3",
                "Charlie",
                "Brown",
                "Tech Corp",
                "Developer",
                vec!["rust".to_string()],
                Some(4.5),
                PriceRange { from: 100, to: 200 },
            ),
            create_sample_auditor(
                "user4",
                "David",
                "Wilson",
                "Tech Corp",
                "Developer",
                vec!["rust".to_string()],
                Some(4.5),
                PriceRange { from: 100, to: 200 },
            ),
            create_sample_auditor(
                "user5",
                "Eve",
                "Davis",
                "Tech Corp",
                "Developer",
                vec!["rust".to_string()],
                Some(4.5),
                PriceRange { from: 100, to: 200 },
            ),
        ];

        // Add auditors to index
        index.add_or_update(auditors.clone()).await?;

        // Test with limit only
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
                free_at_range: None,
                offset: None,
                limit: Some(3),
            })
            .await?;
        assert_eq!(results.len(), 3);

        // Test with offset only
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
                free_at_range: None,
                offset: Some(2),
                limit: None,
            })
            .await?;
        assert_eq!(results.len(), 3); // Should return remaining 3 auditors

        // Test with both offset and limit
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
                free_at_range: None,
                offset: Some(1),
                limit: Some(2),
            })
            .await?;
        assert_eq!(results.len(), 2);

        // Test offset beyond available results
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
                free_at_range: None,
                offset: Some(5),
                limit: None,
            })
            .await?;
        assert_eq!(results.len(), 0);

        // Test pagination with filters
        let results = index
            .search(SearchFilter {
                query: None,
                first_name: None,
                last_name: None,
                full_name: None,
                company: Some("Tech Corp".to_string()),
                tags: Some(vec!["rust".to_string()]),
                price: None,
                price_range: None,
                rating: None,
                free_at_range: None,
                offset: Some(2),
                limit: Some(2),
            })
            .await?;
        assert_eq!(results.len(), 2);
        // Verify that all results have correct company and tags
        for result in &results {
            assert_eq!(result.company, "Tech Corp");
            assert!(result.tags.contains(&"rust".to_string()));
        }
        // Verify that user_ids are unique
        let mut user_ids: Vec<_> = results.iter().map(|r| &r.user_id).collect();
        user_ids.sort();
        user_ids.dedup();
        assert_eq!(user_ids.len(), results.len());

        Ok(())
    }
}