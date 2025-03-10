use std::path::Path;
use regex;
use tantivy::{
    collector::{Count, TopDocs},
    doc,
    query::{BooleanQuery, Occur, QueryParser, RangeQuery, TermQuery, RegexQuery},
    schema::{
        Field, IndexRecordOption, Schema, Value, FAST, INDEXED, STORED, TEXT, STRING,
    },
    Index, IndexReader, IndexWriter, Searcher, TantivyDocument, Term,
};

use crate::{
    error::{ServiceError, ServiceResult},
    models::{PriceRangeFilter, RangeFilter, SearchQuery, SortOption, EntityKind},
};

use common::entities::{
    auditor::Auditor,
    badge::Badge,
    customer::Customer,
    project::Project,
};

pub struct SearchIndex {
    index: Index,
    reader: IndexReader,
    writer: IndexWriter,
    schema: Schema,
    fields: IndexFields,
}

#[derive(Clone)]
pub struct IndexFields {
    user_id: Field,
    avatar: Field,
    first_name: Field,
    last_name: Field,
    about: Field,
    company: Field,
    free_at: Field,
    tags: Field,
    price_from: Field,
    price_to: Field,
    rating: Field,
    last_modified: Field,
    entity_kind: Field,
    published: Field,
}

impl SearchIndex {
    pub fn new<P: AsRef<Path>>(path: P) -> ServiceResult<Self> {
        let schema = Self::create_schema();
        let fields = Self::get_fields(&schema);

        let index = Index::create_in_dir(path, schema.clone())?;
        let reader = index.reader()?;
        let writer = index.writer(50_000_000)?; // 50MB buffer

        Ok(Self {
            index,
            reader,
            writer,
            schema,
            fields,
        })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> ServiceResult<Self> {
        let index = Index::open_in_dir(path)?;
        let schema = index.schema();
        let fields = Self::get_fields(&schema);
        let reader = index.reader()?;
        let writer = index.writer(50_000_000)?; // 50MB buffer

        Ok(Self {
            index,
            reader,
            writer,
            schema,
            fields,
        })
    }

    fn create_schema() -> Schema {
        let mut schema_builder = Schema::builder();

        // Add fields to schema
        schema_builder.add_text_field("user_id", TEXT | STORED);
        schema_builder.add_text_field("avatar", STORED);
        schema_builder.add_text_field("first_name", TEXT | STORED);
        schema_builder.add_text_field("last_name", TEXT | STORED);
        schema_builder.add_text_field("about", TEXT | STORED);
        schema_builder.add_text_field("company", TEXT | STORED);
        schema_builder.add_text_field("free_at", TEXT | STORED);
        schema_builder.add_text_field("tags", TEXT | STORED);
        schema_builder.add_i64_field("price_from", INDEXED | STORED | FAST);
        schema_builder.add_i64_field("price_to", INDEXED | STORED | FAST);
        schema_builder.add_f64_field("rating", INDEXED | STORED | FAST);
        schema_builder.add_i64_field("last_modified", INDEXED | STORED | FAST);
        schema_builder.add_text_field("entity_kind", STRING | STORED);
        schema_builder.add_bool_field("published", INDEXED | STORED);

        schema_builder.build()
    }

    fn get_fields(schema: &Schema) -> IndexFields {
        IndexFields {
            user_id: schema.get_field("user_id").unwrap(),
            avatar: schema.get_field("avatar").unwrap(),
            first_name: schema.get_field("first_name").unwrap(),
            last_name: schema.get_field("last_name").unwrap(),
            about: schema.get_field("about").unwrap(),
            company: schema.get_field("company").unwrap(),
            free_at: schema.get_field("free_at").unwrap(),
            tags: schema.get_field("tags").unwrap(),
            price_from: schema.get_field("price_from").unwrap(),
            price_to: schema.get_field("price_to").unwrap(),
            rating: schema.get_field("rating").unwrap(),
            last_modified: schema.get_field("last_modified").unwrap(),
            entity_kind: schema.get_field("entity_kind").unwrap(),
            published: schema.get_field("published").unwrap(),
        }
    }

    pub fn index_auditor(&mut self, auditor: &Auditor<String>) -> ServiceResult<()> {
        let mut doc = doc!(
            self.fields.user_id => auditor.user_id.to_string(),
            self.fields.avatar => auditor.avatar.clone(),
            self.fields.first_name => auditor.first_name.to_lowercase(),
            self.fields.last_name => auditor.last_name.to_lowercase(),
            self.fields.about => auditor.about.clone(),
            self.fields.company => auditor.company.clone(),
            self.fields.free_at => auditor.free_at.clone(),
            self.fields.price_from => auditor.price_range.from as i64,
            self.fields.price_to => auditor.price_range.to as i64,
            self.fields.rating => auditor.rating.unwrap_or(0.0) as f64,
            self.fields.last_modified => auditor.last_modified,
            self.fields.entity_kind => "auditor"
        );

        for tag in &auditor.tags {
            doc.add_text(self.fields.tags, &tag.to_lowercase());
        }

        self.writer.add_document(doc)?;
        Ok(())
    }

    pub fn index_badge(&mut self, badge: &Badge<String>) -> ServiceResult<()> {
        let mut doc = doc!(
            self.fields.user_id => badge.user_id.to_string(),
            self.fields.avatar => badge.avatar.clone(),
            self.fields.first_name => badge.first_name.to_lowercase(),
            self.fields.last_name => badge.last_name.to_lowercase(),
            self.fields.about => badge.about.clone(),
            self.fields.company => badge.company.clone(),
            self.fields.free_at => badge.free_at.clone(),
            self.fields.price_from => badge.price_range.from as i64,
            self.fields.price_to => badge.price_range.to as i64,
            self.fields.rating => 0.0f64,
            self.fields.last_modified => badge.last_modified,
            self.fields.entity_kind => "badge"
        );

        for tag in &badge.tags {
            doc.add_text(self.fields.tags, &tag.to_lowercase());
        }

        self.writer.add_document(doc)?;
        Ok(())
    }

    pub fn index_customer(&mut self, customer: &Customer<String>) -> ServiceResult<()> {
        let mut doc = doc!(
            self.fields.user_id => customer.user_id.to_string(),
            self.fields.avatar => customer.avatar.clone(),
            self.fields.first_name => customer.first_name.to_lowercase(),
            self.fields.last_name => customer.last_name.to_lowercase(),
            self.fields.about => customer.about.clone(),
            self.fields.company => customer.company.clone(),
            self.fields.free_at => "",
            self.fields.price_from => 0i64,
            self.fields.price_to => 0i64,
            self.fields.rating => customer.rating.unwrap_or(0.0) as f64,
            self.fields.last_modified => customer.last_modified,
            self.fields.entity_kind => "customer"
        );

        for tag in &customer.tags {
            doc.add_text(self.fields.tags, &tag.to_lowercase());
        }

        self.writer.add_document(doc)?;
        Ok(())
    }

    pub fn index_project(&mut self, project: &Project<String>) -> ServiceResult<()> {
        let mut doc = doc!(
            self.fields.user_id => project.id.to_string(),
            self.fields.avatar => "",
            self.fields.first_name => project.name.to_lowercase(),
            self.fields.last_name => "",
            self.fields.about => project.description.clone(),
            self.fields.company => "",
            self.fields.free_at => "",
            self.fields.price_from => project.price.unwrap_or(0i64),
            self.fields.price_to => project.total_cost.unwrap_or(0i64),
            self.fields.rating => 0.0f64,
            self.fields.last_modified => project.last_modified,
            self.fields.entity_kind => "project",
            self.fields.published => project.publish_options.publish
        );

        for tag in &project.tags {
            doc.add_text(self.fields.tags, &tag.to_lowercase());
        }

        self.writer.add_document(doc)?;
        Ok(())
    }

    pub fn commit(&mut self) -> ServiceResult<()> {
        self.writer.commit()?;
        Ok(())
    }

    pub fn delete_by_id(&mut self, id: &str) -> ServiceResult<()> {
        let term = Term::from_field_text(self.fields.user_id, id);
        self.writer.delete_term(term);
        Ok(())
    }

    pub fn search(&self, query: &SearchQuery) -> ServiceResult<(Vec<String>, usize)> {
        let mut subqueries: Vec<(Occur, Box<dyn tantivy::query::Query>)> = Vec::new();

        // Full-text search across multiple fields
        if let Some(text) = &query.text {
            let text = text.to_lowercase();
            let fields = vec![
                    self.fields.first_name,
                    self.fields.last_name,
                    self.fields.about,
                    self.fields.company,
                    self.fields.tags,
                ];
            
            if query.partial_match.unwrap_or(false) {
                let mut text_queries: Vec<(Occur, Box<dyn tantivy::query::Query>)> = Vec::new();
                
                for &field in &fields {
                    let regex_query = RegexQuery::from_pattern(
                        &format!(".*{}.*", regex::escape(&text)),
                        field,
                    )?;
                    
                    text_queries.push((Occur::Should, Box::new(regex_query)));
                }
                
                let bool_query = BooleanQuery::new(text_queries);
                subqueries.push((Occur::Must, Box::new(bool_query)));
            } else {
                let parser = QueryParser::for_index(
                    &self.index,
                    fields,
                );
                
                if let Ok(text_query) = parser.parse_query(&text) {
                    subqueries.push((Occur::Must, text_query));
                }
            }
        }

        // Add filters
        self.add_name_filter(&mut subqueries, &query.name, query.partial_match.unwrap_or(false))?;
        self.add_company_filter(&mut subqueries, &query.company)?;
        self.add_free_at_filter(&mut subqueries, &query.free_at)?;
        self.add_tags_filter(&mut subqueries, &query.tags)?;
        self.add_price_range_filter(&mut subqueries, &query.price_range())?;
        self.add_rating_filter(&mut subqueries, &query.rating())?;
        self.add_entity_kind_filter(&mut subqueries, &query.kind)?;

        if query.kind.contains(&EntityKind::Project) {
            let term_query = TermQuery::new(
                Term::from_field_bool(self.fields.published, true),
                IndexRecordOption::Basic,
            );
            subqueries.push((Occur::Must, Box::new(term_query)));
        }

        let boolean_query = BooleanQuery::new(subqueries);
        let searcher: Searcher = self.reader.searcher();
        let limit = query.per_page.unwrap_or(10) as usize;
        let offset = ((query.page.unwrap_or(1) - 1) * query.per_page.unwrap_or(10)) as usize;

        let all_matching_docs = searcher.search(&boolean_query, &TopDocs::with_limit(1000))?;
        let total = searcher.search(&boolean_query, &Count)?;

        let mut docs = Vec::with_capacity(all_matching_docs.len());
        for (score, doc_address) in all_matching_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address).map_err(|e| {
                ServiceError::Internal(format!("Failed to retrieve document: {:?}", e))
            })?;
            
            if let Some(field_value) = retrieved_doc.get_first(self.fields.user_id) {
                if let Some(id_str) = field_value.as_str() {
                    let price = retrieved_doc.get_first(self.fields.price_from)
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    
                    let rating = retrieved_doc.get_first(self.fields.rating)
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);

                    let kind = retrieved_doc.get_first(self.fields.entity_kind)
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    
                    docs.push((id_str.to_string(), price, rating, score, kind.to_string()));
                }
            }
        }

        if let Some(sort_option) = &query.sort {
            match sort_option {
                SortOption::Relevance => {
                    docs.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
                },
                SortOption::PriceAsc => {
                    docs.sort_by(|a, b| a.1.cmp(&b.1));
                },
                SortOption::PriceDesc => {
                    docs.sort_by(|a, b| b.1.cmp(&a.1));
                },
                SortOption::RatingAsc => {
                    docs.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
                },
                SortOption::RatingDesc => {
                    docs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
                },
            }
        } else {
            docs.sort_by(|a, b| {
                let a_kind_pos = query.kind.iter()
                    .position(|k| k.to_string() == a.4)
                    .unwrap_or(usize::MAX);
                let b_kind_pos = query.kind.iter()
                    .position(|k| k.to_string() == b.4)
                    .unwrap_or(usize::MAX);

                match a_kind_pos.cmp(&b_kind_pos) {
                    std::cmp::Ordering::Equal => {
                        b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal)
                    },
                    other => other
                }
            });
        }

        let start = offset.min(docs.len());
        let end = (offset + limit).min(docs.len());
        let paginated_docs = &docs[start..end];
        let ids: Vec<String> = paginated_docs.iter().map(|(id, _, _, _, _)| id.clone()).collect();

        Ok((ids, total))
    }

    fn add_name_filter(
        &self,
        subqueries: &mut Vec<(Occur, Box<dyn tantivy::query::Query>)>,
        name: &Option<String>,
        partial_match: bool,
    ) -> ServiceResult<()> {
        if let Some(name) = name {
            let name = name.to_lowercase();
            let mut name_queries: Vec<(Occur, Box<dyn tantivy::query::Query>)> = Vec::new();

            if partial_match {
                let first_name_query = RegexQuery::from_pattern(
                    &format!("{}.*", regex::escape(&name)),
                    self.fields.first_name,
                )?;
                let last_name_query = RegexQuery::from_pattern(
                    &format!("{}.*", regex::escape(&name)),
                    self.fields.last_name,
                )?;
                
                name_queries.push((Occur::Should, Box::new(first_name_query)));
                name_queries.push((Occur::Should, Box::new(last_name_query)));
            } else {
                let first_name_query = TermQuery::new(
                    Term::from_field_text(self.fields.first_name, &name),
                    IndexRecordOption::Basic,
                );
                let last_name_query = TermQuery::new(
                    Term::from_field_text(self.fields.last_name, &name),
                    IndexRecordOption::Basic,
                );
                
                name_queries.push((Occur::Should, Box::new(first_name_query)));
                name_queries.push((Occur::Should, Box::new(last_name_query)));
            }

            let bool_query = BooleanQuery::new(name_queries);
            subqueries.push((Occur::Must, Box::new(bool_query)));
        }
        Ok(())
    }

    fn add_company_filter(
        &self,
        subqueries: &mut Vec<(Occur, Box<dyn tantivy::query::Query>)>,
        company: &Option<String>,
    ) -> ServiceResult<()> {
        if let Some(company) = company {
            let term_query = TermQuery::new(
                Term::from_field_text(self.fields.company, company),
                IndexRecordOption::Basic,
            );
            subqueries.push((Occur::Must, Box::new(term_query)));
        }
        Ok(())
    }

    fn add_free_at_filter(
        &self,
        subqueries: &mut Vec<(Occur, Box<dyn tantivy::query::Query>)>,
        free_at: &Option<String>,
    ) -> ServiceResult<()> {
        if let Some(date) = free_at {
            let term_query = TermQuery::new(
                Term::from_field_text(self.fields.free_at, date),
                IndexRecordOption::Basic,
            );
            subqueries.push((Occur::Must, Box::new(term_query)));
        }
        Ok(())
    }

    fn add_price_range_filter(
        &self,
        subqueries: &mut Vec<(Occur, Box<dyn tantivy::query::Query>)>,
        price_range: &Option<PriceRangeFilter>,
    ) -> ServiceResult<()> {
        if let Some(range) = price_range {
            if let Some(from) = range.from {
                let range_query = RangeQuery::new_i64(
                    self.schema.get_field_name(self.fields.price_from).to_string(),
                    from..i64::MAX,
                );
                subqueries.push((Occur::Must, Box::new(range_query)));
            }
            if let Some(to) = range.to {
                let range_query = RangeQuery::new_i64(
                    self.schema.get_field_name(self.fields.price_to).to_string(),
                    i64::MIN..to,
                );
                subqueries.push((Occur::Must, Box::new(range_query)));
            }
        }
        Ok(())
    }

    fn add_rating_filter(
        &self,
        subqueries: &mut Vec<(Occur, Box<dyn tantivy::query::Query>)>,
        rating: &Option<RangeFilter<f32>>,
    ) -> ServiceResult<()> {
        if let Some(range) = rating {
            if let Some(from) = range.from {
                let range_query = RangeQuery::new_f64(
                    self.schema.get_field_name(self.fields.rating).to_string(),
                    from as f64..f64::MAX,
                );
                subqueries.push((Occur::Must, Box::new(range_query)));
            }
            if let Some(to) = range.to {
                let range_query = RangeQuery::new_f64(
                    self.schema.get_field_name(self.fields.rating).to_string(),
                    f64::MIN..to as f64,
                );
                subqueries.push((Occur::Must, Box::new(range_query)));
            }
        }
        Ok(())
    }

    fn add_tags_filter(
        &self,
        subqueries: &mut Vec<(Occur, Box<dyn tantivy::query::Query>)>,
        tags: &Option<Vec<String>>,
    ) -> ServiceResult<()> {
        if let Some(tags) = tags {
            let mut tag_queries: Vec<(Occur, Box<dyn tantivy::query::Query>)> = Vec::new();
            
            for tag in tags {
                let term_query = TermQuery::new(
                    Term::from_field_text(self.fields.tags, &tag.to_lowercase()),
                    IndexRecordOption::Basic,
                );
                tag_queries.push((Occur::Must, Box::new(term_query)));
            }
            
            if !tag_queries.is_empty() {
                let bool_query = BooleanQuery::new(tag_queries);
                subqueries.push((Occur::Must, Box::new(bool_query)));
            }
        }
        Ok(())
    }

    fn add_entity_kind_filter(
        &self,
        subqueries: &mut Vec<(Occur, Box<dyn tantivy::query::Query>)>,
        kinds: &[EntityKind],
    ) -> ServiceResult<()> {
        if !kinds.is_empty() {
            let mut kind_queries: Vec<(Occur, Box<dyn tantivy::query::Query>)> = Vec::new();
            
            for kind in kinds {
                let kind_str = kind.to_string().to_lowercase();
                let term_query = TermQuery::new(
                    Term::from_field_text(self.fields.entity_kind, &kind_str),
                    IndexRecordOption::Basic,
                );
                kind_queries.push((Occur::Should, Box::new(term_query)));
            }
            
            if !kind_queries.is_empty() {
                let bool_query = BooleanQuery::new(kind_queries);
                subqueries.push((Occur::Must, Box::new(bool_query)));
            }
        } else {
            tracing::warn!("No entity kinds specified for filtering");
        }
        Ok(())
    }
}

impl ToString for EntityKind {
    fn to_string(&self) -> String {
        match self {
            EntityKind::Auditor => "auditor".to_string(),
            EntityKind::Badge => "badge".to_string(),
            EntityKind::Customer => "customer".to_string(),
            EntityKind::Project => "project".to_string(),
        }
    }
}
