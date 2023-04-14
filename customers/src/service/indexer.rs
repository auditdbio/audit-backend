use common::context::Context;
use mongodb::bson::Document;

pub struct IndexerService {
    context: Context,
}

impl IndexerService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn index(&self, since: i64) -> anyhow::Result<Vec<Document>> {
        todo!()
    }
}
