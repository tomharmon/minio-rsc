mod list_buckets_response;
mod list_objects_response;
pub use list_buckets_response::*;
pub use list_objects_response::*;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Owner {
    pub display_name: String,
    pub id: String,
}
