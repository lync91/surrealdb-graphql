mod tickets_query;

use async_graphql::Object;
use tickets_query::TicketsQuery;

pub struct QueryRoot;
#[Object]
impl QueryRoot {
    /// API version - this is visible in the gql doc!
    async fn version(&self) -> &str {
        "1.0"
    }

    async fn tickets(&self) -> TicketsQuery {
        TicketsQuery
    }
}
