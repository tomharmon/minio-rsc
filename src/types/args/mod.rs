mod list_objects_args;

pub use list_objects_args::*;

use super::QueryMap;

pub(crate) trait BaseArgs {
    fn extra_query_map(&self) -> QueryMap {
        QueryMap::default()
    }

    fn extra_headers(&self) -> bool {
        false
    }
}
