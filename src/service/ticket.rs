use std::borrow::BorrowMut;

use crate::{
    ctx::Ctx,
    error::{ApiError, ApiResult, Error},
    Db, graphql,
};
use async_graphql::{ComplexObject, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct Sale {
    #[graphql(skip)]
    id: Option<Thing>,
    ticket: Option<Ticket>,
    user: String
}



#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
struct Detail {
    c: i32,
    d: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct Ticket {
    #[graphql(skip)]
    pub id: Option<Thing>,
    pub creator: String,
    pub title: String,
    detail: Vec<Detail>,
}

#[ComplexObject]
impl Sale {
    async fn id(&self) -> String {
        self.id.as_ref().map(|t| &t.id).expect("id").to_raw()
    }
}

#[ComplexObject]
impl Ticket {
    async fn id(&self) -> String {
        self.id.as_ref().map(|t| &t.id).expect("id").to_raw()
    }
}

#[derive(Deserialize, InputObject)]
pub struct CreateTicketInput {
    pub title: String,
}
#[derive(Serialize)]
pub struct CreateSaleInput {
    id: Option<Thing>,
    ticket: Option<Thing>,
    user: String
}

#[derive(Deserialize, InputObject)]
pub struct CreateTestInput {
    pub c: i32,
    pub d: i32
}


pub struct TicketService<'a> {
    pub db: &'a Db,
    pub ctx: &'a Ctx,
}
impl<'a> TicketService<'a> {
    pub async fn list_tickets(&self) -> ApiResult<Vec<Ticket>> {
        self.db
            .select("tickets")
            .await
            .map_err(ApiError::from(self.ctx))
    }

    pub async fn list_sales(&self) -> ApiResult<String> {
        let sale = self.db
            .query("select * from sales fetch ticket")
            .await
            .map(|mut res| res.take(0))
            .map(|s| s.unwrap())
            .map_err(ApiError::from(self.ctx));
        Ok("sale".to_string())
    }

    pub async fn sale_relate(&self) -> ApiResult<Vec<Sale>> {
        let sale = self.db
            .query("select in from sale_relate")
            .await
            .map(|mut res| res.take(0))
            .map(|s| s.unwrap())
            .map_err(ApiError::from(self.ctx));
        sale
    }

    pub async fn create_ticket(&self, ct_input: CreateTicketInput, test_input: Vec<CreateTestInput>) -> ApiResult<Ticket> {
        let ticket = 
        self.db
            .create("tickets")
            .content(Ticket {
                id: None,
                creator: self.ctx.user_id()?,
                title: ct_input.title,
                detail: test_input.into_iter().map(|item| Detail {
                    c: item.c,
                    d: item.d
                }).collect()
            })
            .await
            .map(|v: Vec<Ticket>| v.into_iter().next().expect("created ticket"))
            .map_err(ApiError::from(self.ctx));
        let ticket_id = ticket.as_ref().unwrap().clone().id;
        let sale = self.db
        .create("sales")
        .content(CreateSaleInput {
            id: None,
            ticket: ticket_id.clone(),
            user: "Lync".to_string()
        })
        .await
        .map(|s: Vec<Sale>| s.into_iter().next().expect("sale created"));
        let sale_id = sale.as_ref().unwrap().clone().id;
        // let sale_id_str: String = sale_id.;
        let query_str = format!("RELATE {}:{}->sale_relate->{}:{}", ticket_id.as_ref().unwrap().clone().tb, &ticket_id.unwrap().id, sale_id.as_ref().unwrap().clone().tb, sale_id.as_ref().unwrap().clone().id);
        self.db.query(query_str).await;
        ticket
    }

    pub async fn delete_ticket(&self, id: String) -> ApiResult<Ticket> {
        // NOTE: If the input is parsed from Thing format
        // let t = thing(&id).map_err(|e| ApiError {
        //     req_id: self.ctx.req_id(),
        //     error: Error::SurrealDbParse {
        //         source: e.to_string(),
        //         id: id.clone(),
        //     },
        // })?;
        match self
            .db
            // .delete(t)
            .delete(("tickets", &id))
            .await
        {
            Ok(option) => option.ok_or(ApiError {
                req_id: self.ctx.req_id(),
                error: Error::SurrealDbNoResult {
                    source: "none".to_string(),
                    id,
                },
            }),
            Err(e) => Err(ApiError {
                req_id: self.ctx.req_id(),
                error: Error::SurrealDbNoResult {
                    source: e.to_string(),
                    id,
                },
            }),
        }
    }
}
