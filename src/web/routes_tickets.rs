use axum::extract::{FromRef, Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};

use crate::ctx::Ctx;
use crate::model::{ModelController, ModelManager, Ticket, TicketForCreate};
use crate::Result;

// FIXME: WATCH this for a great explanation of ModelControllers vs ModelManager pattern:
// REF: https://youtu.be/JdLi69mWIIE?list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q
// TL;DR:
// ModelControllers (Stateless! Does the work)
// ModelManager (DBPool State! Gives the resources)
//
// FIXME: After adding Ctx (see mw_auth.rs), we now pass Ctx to our handlers
// for privileges and access level control at both web and model layers.

// Return the Router so can merge with main. Gonna be nested at "/api"
// This time we pass the state ModelController to the Router
// NOTE: Use with_state(mc) to pass ONE state through all routes
// NOTE: To pass MULTIPLE states, create an AppState struct (see above)
pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/tickets", post(create_ticket).get(list_tickets))
        .route("/tickets/:id", delete(delete_ticket))
        .with_state(mm) // Originally 'mc'
}

// region: -- REST Handlers
// NOTE: State is an Axum extractor struct. State is app state that's shared across all handlers.
// State will be similar to other extractor structs.
// NOTE: State(mm) and Json(ticket_fc) is DESTRUCTURING
// NOTE: Recall that ModelController has impl CRUD functionality
async fn create_ticket(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Json(ticket_fc): Json<TicketForCreate>,
) -> Result<Json<Ticket>> {
    println!("->> {:<12} - create_ticket", "HANDLER");

    let ticket = mm.mc.create_ticket(ctx, ticket_fc).await?;

    Ok(Json(ticket))
}

async fn list_tickets(State(mm): State<ModelManager>, ctx: Ctx) -> Result<Json<Vec<Ticket>>> {
    println!("->> {:<12} - list_tickets", "HANDLER");

    let tickets = mm.mc.list_tickets(ctx).await?;

    Ok(Json(tickets))
}

async fn delete_ticket(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Path(id): Path<u64>,
) -> Result<Json<Ticket>> {
    println!("->> {:<12} - delete_ticket", "HANDLER");

    let ticket = mm.mc.delete_ticket(ctx, id).await?;

    Ok(Json(ticket))
}

// endregion: -- REST Handlers
