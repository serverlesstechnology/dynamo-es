use std::marker::PhantomData;

use async_trait::async_trait;
use cqrs_es::{Aggregate, View};
use persist_es::{PersistenceError, QueryContext, ViewRepository};

/// A DynamoDb backed query repository for use in backing a `GenericQuery`.
pub struct DynamoViewRepository<V, A> {
    _phantom: PhantomData<(V, A)>,
}

#[async_trait]
impl<V, A> ViewRepository<V, A> for DynamoViewRepository<V, A>
where
    V: View<A>,
    A: Aggregate,
{
    async fn load(
        &self,
        _query_instance_id: &str,
    ) -> Result<Option<(V, QueryContext)>, PersistenceError> {
        todo!()
    }

    async fn update_view(&self, _view: V, _context: QueryContext) -> Result<(), PersistenceError> {
        todo!()
    }
}
