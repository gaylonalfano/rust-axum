use crate::{Error, Result}; // NOTE: Eventually can create custom Model Error
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex}; // In-memory store

// region -- Ticket (Task) Types
// Clone so we can save and send copy back to client
// Serialize to JSON to send back to client
#[derive(Clone, Debug, Serialize)]
pub struct Ticket {
    pub id: u64,
    pub cid: u64, // creator user_id (from ctx.user_id())
    pub title: String,
}

#[derive(Deserialize)]
pub struct TicketForCreate {
    pub title: String,
}
// end region -- Ticket (Task) Types
