use chrono::Utc;
use jsonwebtoken::{decode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use mongodb::bson::oid::ObjectId;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{
    constants::DURATION,
    entities::{
        auditor::{Auditor, PublicAuditor},
        contacts::Contacts,
        customer::{Customer, PublicCustomer},
        project::{Project, PublicProject},
    },
    error::{self, AddCode},
};

pub static ENCODING_KEY: Lazy<EncodingKey> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    EncodingKey::from_secret(secret.as_bytes())
});

pub static DECODING_KEY: Lazy<DecodingKey> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    DecodingKey::from_secret(secret.as_bytes())
});

#[derive(Debug, Clone, PartialEq)]
pub enum Auth {
    Service(String, bool),
    Admin(ObjectId),
    User(ObjectId),
    None,
}

impl Auth {
    pub fn id(&self) -> Option<&ObjectId> {
        match self {
            Auth::Admin(id) => Some(id),
            Auth::User(id) => Some(id),
            _ => None,
        }
    }

    pub fn authorized(self) -> Self {
        match self {
            Auth::Service(name, _) => Auth::Service(name, true),
            a => a,
        }
    }

    pub fn full_access(&self) -> bool {
        match self {
            Auth::Admin(_) => true,
            Auth::Service(name, _) => name != "search",
            _ => false,
        }
    }

    pub fn public_customer(&self, customer: Customer<ObjectId>) -> PublicCustomer {
        let mut contacts = Contacts {
            telegram: None,
            email: None,
            public_contacts: false,
        };

        if customer.contacts.public_contacts || self.full_access() {
            contacts = customer.contacts;
        }

        if &Auth::None == self || &Auth::Service("search".to_string(), false) == self {
            contacts.telegram = None;
            contacts.email = None;
        }

        PublicCustomer {
            user_id: customer.user_id.to_hex(),
            avatar: customer.avatar,
            first_name: customer.first_name,
            last_name: customer.last_name,
            about: customer.about,
            company: customer.company,
            contacts,
            tags: customer.tags,
        }
    }

    pub fn public_auditor(&self, auditor: Auditor<ObjectId>) -> PublicAuditor {
        let mut contacts = Contacts {
            telegram: None,
            email: None,
            public_contacts: false,
        };

        if auditor.contacts.public_contacts || self.full_access() {
            contacts = auditor.contacts;
        }

        if &Auth::None == self || &Auth::Service("search".to_string(), false) == self {
            contacts.telegram = None;
            contacts.email = None;
        }

        PublicAuditor {
            user_id: auditor.user_id.to_hex(),
            avatar: auditor.avatar,
            first_name: auditor.first_name,
            last_name: auditor.last_name,
            about: auditor.about,
            company: auditor.company,
            contacts,
            free_at: auditor.free_at,
            price_range: auditor.price_range,
            tags: auditor.tags,
        }
    }

    pub fn public_project(&self, project: Project<ObjectId>) -> PublicProject {
        let mut contacts = Contacts {
            telegram: None,
            email: None,
            public_contacts: false,
        };

        if project.creator_contacts.public_contacts || self.full_access() {
            contacts = project.creator_contacts;
        }

        if &Auth::None == self || &Auth::Service("search".to_string(), false) == self {
            contacts.telegram = None;
            contacts.email = None;
        }

        PublicProject {
            id: project.id.to_hex(),
            customer_id: project.customer_id.to_hex(),
            name: project.name,
            description: project.description,
            scope: project.scope,
            tags: project.tags,
            publish_options: project.publish_options,
            status: project.status,
            creator_contacts: contacts,
            price: project.price,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Role {
    Admin,
    User,
    Service,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    role: Role,
    user_id: Option<String>,
    service_name: Option<String>,
    user_authorized: Option<bool>,
    exp: i64,
}

impl Auth {
    pub fn from_token(token: &str) -> error::Result<Self> {
        match decode::<Claims>(&token, &DECODING_KEY, &Validation::new(Algorithm::HS512)) {
            Ok(c) => {
                let claims = c.claims;
                match claims.role {
                    Role::Admin => {
                        let id = claims.user_id.unwrap().parse()?;
                        Ok(Auth::Admin(id))
                    }
                    Role::User => {
                        let id = claims.user_id.unwrap().parse()?;
                        Ok(Auth::User(id))
                    }
                    Role::Service => {
                        let name = claims.service_name.unwrap();
                        let user_authorized = claims.user_authorized.unwrap_or(false);
                        Ok(Auth::Service(name, user_authorized))
                    }
                }
            }
            Err(err) => {
                panic!("Error: {:?}", err);
            }
        }
    }

    pub fn to_token(&self) -> error::Result<String> {
        let header = Header {
            alg: Algorithm::HS512,
            ..Default::default()
        };
        let exp = Utc::now().timestamp() + DURATION.num_seconds();
        let claims = match self {
            Auth::Service(name, user_auth) => Claims {
                role: Role::Service,
                user_id: None,
                service_name: Some(name.clone()),
                exp,
                user_authorized: Some(*user_auth),
            },
            Auth::Admin(id) => Claims {
                role: Role::Admin,
                user_id: Some(id.to_hex()),
                service_name: None,
                exp,
                user_authorized: None,
            },
            Auth::User(id) => Claims {
                role: Role::User,
                user_id: Some(id.to_hex()),
                service_name: None,
                exp,
                user_authorized: None,
            },
            Auth::None => {
                return Err(anyhow::anyhow!("Cannot create token for Auth::None").code(500))
            }
        };

        let token = match jsonwebtoken::encode(&header, &claims, &ENCODING_KEY) {
            Ok(t) => t,
            Err(_) => return Err(anyhow::anyhow!("Failed to encode token").code(500)),
        };
        Ok(token)
    }
}
