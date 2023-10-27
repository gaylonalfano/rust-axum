// NOTE: Extractors at a high level is something that implements
// FromRequest or FromRequestParts. This allows the extractor to
// take parts (or whole) of the request, and turn into something
// that can appear in the arguments list of a handler and implement
// the whole Axum Handler trait. Jon Gjengset's explanation:
// REF: https://youtu.be/Wnb_n5YktO8?t=3273
// NOTE: Jon believes the "heart of Axum" is the impl_handler macro:
// REF: https://github.com/tokio-rs/axum/blob/276aa9e4b013de1646ea57cfcbf74e5966524f68/axum/src/handler/mod.rs#L206
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
