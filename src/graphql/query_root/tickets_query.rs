use crate::{
    ctx::Ctx,
    service::ticket::{Ticket, TicketService, Sale},
    Db,
};
use async_graphql::{Context, Object, Result};

pub struct TicketsQuery;
#[Object]
impl TicketsQuery {
    async fn list(&self, ctx: &Context<'_>) -> Result<Vec<Ticket>> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<Ctx>()?;
        Ok(TicketService { db, ctx }.list_tickets().await?)
    }
    async fn list_sale(&self, ctx: &Context<'_>) -> Result<Vec<Sale>> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<Ctx>()?;
        Ok(TicketService { db, ctx }.list_sales().await?)
    }
    async fn sale_relate(&self, ctx: &Context<'_>) -> Result<Vec<Sale>> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<Ctx>()?;
        Ok(TicketService { db, ctx }.sale_relate().await?)
    }
}
