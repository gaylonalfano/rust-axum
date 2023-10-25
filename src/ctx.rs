// NOTE: The issue: user_id gets lost a bit in the middleware.
// We also want to probably ensure they cannot list, delete or
// create tickets, and even add the user_id to the created ticket
// is probably ideal. The way to do that is with extractors.
// We're going to create a custom extractor for ctx that will be
// used by all layers (model, web, etc.).
// NOTE: To align with Axum Extractors, we need to impl the Async
// trait on the Extractor (we do this inside the middleware)
#[derive(Clone, Debug)]
pub struct Ctx {
    user_id: u64,
}

impl Ctx {
    // NOTE: user_id is immutable, but we could add
    // mutable props (e.g., access level) later on
    // Constructor:
    pub fn new(user_id: u64) -> Self {
        Self { user_id }
    }

    // Property Accessors:
    pub fn user_id(&self) -> u64 {
        // This way nobody can change user_id of a Ctx that
        // is not within this module
        self.user_id
    }
}
