use crate::ctx::Ctx;
use crate::model::{Ticket, TicketForCreate};
use crate::{Error, Result}; // NOTE: Eventually can create custom Model Error
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex}; // In-memory store

// region: -- Model Controller
#[derive(Clone)]
pub struct ModelController {
    // NOTE: This is ONLY for quick prototyping! The space will grow infinitely!
    // Will have the store (db, sqlx, etc.) embedded inside
    // Clone - clones the Arc (not the Vec!)
    // Mutex - Mutual exclusion protects our Vector by making access to Vec exclusive
    tickets_store: Arc<Mutex<Vec<Option<Ticket>>>>,
}

// Constructor
// NOTE: We could derive the Arc::default directly in the ModelController struct,
// but creating a custom Constructor allows us to define the fn signature.
// This allows us to easily swap different data store implementation.
// REF: https://youtu.be/XZtlD_m59sM?t=1722
impl ModelController {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            tickets_store: Arc::default(),
        })
    }

    // CRUD Implementation
    // NOTE: U: We're adding Ctx to our model layer as well.
    pub async fn create_ticket(&self, ctx: Ctx, ticket_fc: TicketForCreate) -> Result<Ticket> {
        // locking for the mutex to prevent multiple threads accessing same shared resource
        let mut store = self.tickets_store.lock().unwrap();

        // The locked mutex allows us to safely get a new id
        let id = store.len() as u64;
        let ticket = Ticket {
            id,
            cid: ctx.user_id(),
            title: ticket_fc.title,
        };
        // Add to clone/copy store
        store.push(Some(ticket.clone()));

        // Return original ticket with Ok
        Ok(ticket)
    }

    pub async fn list_tickets(&self, _ctx: Ctx) -> Result<Vec<Ticket>> {
        let store = self.tickets_store.lock().unwrap();

        // Clone the Option and its content (excludes deleted items)
        let tickets = store.iter().filter_map(|t| t.clone()).collect();

        Ok(tickets)
    }

    // TODO: Need get_ticket()
    // TODO Need update_ticket()

    pub async fn delete_ticket(&self, _ctx: Ctx, id: u64) -> Result<Ticket> {
        let mut store = self.tickets_store.lock().unwrap();

        // Take the ticket out if found (and no error)
        let ticket = store.get_mut(id as usize).and_then(|t| t.take());

        // Return Error if not found (*updated error.rs)
        ticket.ok_or(Error::TicketDeleteFailIdNotFound { id })
    }
}

// end region: -- Model Controller
