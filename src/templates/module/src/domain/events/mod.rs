// Domain Events Module
// All domain events for the Metaphor bounded context

pub mod metaphor_events;

pub use metaphor_events::{
    MetaphorCreated, MetaphorDeleted, MetaphorMetadataChanged, MetaphorStatusChanged,
    MetaphorTagsChanged, MetaphorUpdated, DomainEvent, EventStore,
};