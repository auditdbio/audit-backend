use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    context::Context,
    entities::{
        audit_request::PriceRange,
        auditor::PublicAuditor,
        bage::{Bage, PublicBage},
        contacts::Contacts,
    },
    error::{self, AddCode},
};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBage {
    avatar: Option<String>,
    first_name: String,
    last_name: String,
    about: Option<String>,
    company: Option<String>,
    contacts: Contacts,
    free_at: Option<String>,
    price_range: Option<PriceRange>,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BageChange {
    avatar: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    about: Option<String>,
    company: Option<String>,
    contacts: Option<Contacts>,
    free_at: Option<String>,
    price_range: Option<PriceRange>,
    tags: Option<Vec<String>>,
}

pub struct BageService {
    context: Context,
}

impl BageService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, bage: CreateBage) -> error::Result<Bage<String>> {
        let auth = self.context.auth();

        let bages = self.context.try_get_repository::<Bage<ObjectId>>()?;

        let bage = Bage {
            user_id: ObjectId::new(),
            avatar: bage.avatar.unwrap_or_default(),
            first_name: bage.first_name,
            last_name: bage.last_name,
            about: bage.about.unwrap_or_default(),
            company: bage.company.unwrap_or_default(),
            contacts: bage.contacts,
            tags: bage.tags.unwrap_or_default(),
            last_modified: Utc::now().timestamp_micros(),
            free_at: bage.free_at.unwrap_or_default(),
            price_range: bage.price_range.unwrap_or_default(),
        };

        bages.insert(&bage).await?;

        Ok(bage.stringify())
    }

    pub async fn find(&self, id: ObjectId) -> error::Result<Option<PublicBage>> {
        let auth = self.context.auth();

        let bages = self.context.try_get_repository::<Bage<ObjectId>>()?;

        let Some(bage) = bages.find("user_id", &Bson::ObjectId(id)).await? else {
            return Ok(None);
        };

        if !Read.get_access(auth, &bage) {
            return Err(anyhow::anyhow!("User is not available to change this bage").code(400));
        }

        Ok(Some(auth.public_bage(bage)))
    }

    pub async fn change(&self, change: BageChange) -> error::Result<Bage<String>> {
        let auth = self.context.auth();
        let id = *auth.id().unwrap();

        let bages = self.context.try_get_repository::<Bage<ObjectId>>()?;

        let Some(mut bage) = bages.find("user_id", &Bson::ObjectId(id)).await? else {
            return Err(anyhow::anyhow!("No bage found").code(400));
        };

        if !Edit.get_access(auth, &bage) {
            return Err(anyhow::anyhow!("User is not available to change this bage").code(400));
        }

        if let Some(avatar) = change.avatar {
            bage.avatar = avatar;
        }

        if let Some(first_name) = change.first_name {
            bage.first_name = first_name;
        }

        if let Some(last_name) = change.last_name {
            bage.last_name = last_name;
        }

        if let Some(about) = change.about {
            bage.about = about;
        }

        if let Some(company) = change.company {
            bage.company = company;
        }

        if let Some(contacts) = change.contacts {
            bage.contacts = contacts;
        }

        if let Some(tags) = change.tags {
            bage.tags = tags;
        }

        if let Some(free_at) = change.free_at {
            bage.free_at = free_at;
        }

        if let Some(price_range) = change.price_range {
            bage.price_range = price_range;
        }

        bage.last_modified = Utc::now().timestamp_micros();

        bages.delete("user_id", &id).await?;
        bages.insert(&bage).await?;

        Ok(bage.stringify())
    }

    pub async fn substitute(
        &self,
        bage_id: ObjectId,
        user_id: ObjectId,
    ) -> error::Result<PublicAuditor> {
        // send code

        todo!()
    }

    pub async fn make_substitute(
        &self,
        bage_id: ObjectId,
        user_id: ObjectId,
        code: String,
    ) -> error::Result<PublicAuditor> {
        // get audit requests
        // delete audit requests
        // delete bage
        // create auditor
        // create audit requests

        todo!()
    }

    pub async fn delete(&self, bage_id: ObjectId) -> error::Result<PublicBage> {
        // send code
        todo!()
    }

    pub async fn make_delete(
        &self,
        bage_id: ObjectId,
        user_id: ObjectId,
        code: String,
    ) -> error::Result<PublicAuditor> {
        // get audit requests
        // delete audit requests
        // delete bage

        todo!()
    }
}
