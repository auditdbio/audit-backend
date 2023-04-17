use anyhow::bail;
use common::{
    context::Context,
    entities::{customer::Customer, project::Project},
};
use mongodb::bson::{oid::ObjectId, Document};

pub struct IndexerService {
    context: Context,
}

impl IndexerService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn index_customer(&self, since: i64) -> anyhow::Result<Vec<Document>> {
        let auth = self.context.auth();

        // TODO: make authentication check

        let Some(customers) = self.context.get_repository::<Customer<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let customers = customers.get_all_since(since).await?;

        Ok(customers
            .into_iter()
            .filter_map(|x| x.into())
            .collect::<Vec<_>>())
    }

    pub async fn index_project(&self, since: i64) -> anyhow::Result<Vec<Document>> {
        let auth = self.context.auth();

        // TODO: make authentication check

        let Some(customers) = self.context.get_repository::<Project<ObjectId>>() else {
            bail!("No project repository found")
        };

        let customers = customers.get_all_since(since).await?;

        Ok(customers
            .into_iter()
            .filter_map(|x| x.into())
            .collect::<Vec<_>>())
    }
}

/*
#[get("/api/customer/data/{resource}/{timestamp}")]
pub async fn get_data(
    _req: HttpRequest,
    since: web::Path<(String, i64)>,
    project_repo: web::Data<ProjectRepo>,
    customer_repo: web::Data<CustomerRepo>,
    _manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let (resource, since) = since.into_inner();
    //let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap
    // if session.role != Role::Service {
    //     return Ok(HttpResponse::Unauthorized().finish());
    // }

    match resource.as_str() {
        "project" => {
            let projects = project_repo.get_all_since(since).await?;
            Ok(HttpResponse::Ok().json(
                projects
                    .into_iter()
                    .map(Project::stringify)
                    .map(Project::to_doc)
                    .collect::<Vec<_>>(),
            ))
        }
        "customer" => {
            let customers = customer_repo.get_all_since(since).await?;
            Ok(HttpResponse::Ok().json(
                customers
                    .into_iter()
                    .map(Customer::stringify)
                    .map(Customer::to_doc)
                    .collect::<Vec<_>>(),
            ))
        }
        _ => Ok(HttpResponse::NotFound().finish()),
    }
}
 */
