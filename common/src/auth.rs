use anyhow::bail;
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
    },
};

pub static ENCODING_KEY: Lazy<EncodingKey> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    EncodingKey::from_secret(secret.as_bytes())
});

pub static DECODING_KEY: Lazy<DecodingKey> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    DecodingKey::from_secret(secret.as_bytes())
});

#[derive(Debug, Clone)]
pub enum Auth {
    Service(String),
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

    pub fn full_access(&self) -> bool {
        match self {
            Auth::Admin(_) | Auth::Service(_) => true,
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
    exp: i64,
}

impl Auth {
    pub fn from_token(token: &str) -> anyhow::Result<Self> {
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
                        Ok(Auth::Service(name))
                    }
                }
            }
            Err(err) => {
                panic!("Error: {:?}", err);
            }
        }
    }

    pub fn to_token(&self) -> anyhow::Result<String> {
        let header = Header {
            alg: Algorithm::HS512,
            ..Default::default()
        };
        let exp = Utc::now().timestamp() + DURATION.num_seconds();
        let claims = match self {
            Auth::Service(name) => Claims {
                role: Role::Service,
                user_id: None,
                service_name: Some(name.clone()),
                exp,
            },
            Auth::Admin(id) => Claims {
                role: Role::Admin,
                user_id: Some(id.to_hex()),
                service_name: None,
                exp,
            },
            Auth::User(id) => Claims {
                role: Role::User,
                user_id: Some(id.to_hex()),
                service_name: None,
                exp,
            },
            Auth::None => bail!("Cannot create token for Auth::None"),
        };

        let token = match jsonwebtoken::encode(&header, &claims, &ENCODING_KEY) {
            Ok(t) => t,
            Err(_) => bail!("Failed to encode token"),
        };
        Ok(token)
    }
}
