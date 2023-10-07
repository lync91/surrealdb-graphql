mod tickets_mutation;
use async_graphql::Object;
use tickets_mutation::TicketsMutation;

pub struct MutationRoot;
#[Object]
impl MutationRoot {
    async fn tickets(&self) -> TicketsMutation {
        TicketsMutation
    }
}
