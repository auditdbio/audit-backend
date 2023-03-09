use utoipa::{ToSchema};

pub struct Id();

impl<'s> ToSchema<'s> for Id {
    fn schema() -> (&'s str, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>) {
        (
            "Id",
            utoipa::openapi::ObjectBuilder::new()
                        .schema_type(utoipa::openapi::SchemaType::String)
                        .into()
        )
    }
}
