use crate::DynamoEventRepository;
use cqrs_es::CqrsFramework;
use persist_es::{PersistedEventStore, PersistedSnapshotStore};

/// A convenience type for a CqrsFramework backed by
/// [DynamoStore](struct.DynamoStore.html).
pub type DynamoCqrs<A> = CqrsFramework<A, PersistedEventStore<DynamoEventRepository, A>>;

/// A convenience type for a CqrsFramework backed by
/// [DynamoSnapshotStore](struct.DynamoSnapshotStore.html).
pub type DynamoSnapshotCqrs<A> = CqrsFramework<A, PersistedSnapshotStore<DynamoEventRepository, A>>;
