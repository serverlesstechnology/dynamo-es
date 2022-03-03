use std::sync::Arc;

use cqrs_es::persist::PersistedEventStore;
use cqrs_es::{Aggregate, CqrsFramework, Query};

use crate::{DynamoCqrs, DynamoEventRepository};

/// A convenience function for creating a CqrsFramework from a DynamoDb client
/// and queries.
pub fn dynamodb_cqrs<A>(
    dynamo_client: aws_sdk_dynamodb::client::Client,
    query_processor: Vec<Arc<dyn Query<A>>>,
) -> DynamoCqrs<A>
where
    A: Aggregate,
{
    let repo = DynamoEventRepository::new(dynamo_client);
    let store = PersistedEventStore::new(repo);
    CqrsFramework::new(store, query_processor)
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::cqrs::dynamodb_cqrs;
    use crate::testing::tests::{test_dynamodb_client, TestQueryRepository};
    use crate::DynamoViewRepository;

    #[tokio::test]
    async fn test_valid_cqrs_framework() {
        let client = test_dynamodb_client().await;
        let view_repo = DynamoViewRepository::new("test_query", client.clone());
        let query = TestQueryRepository::new(view_repo);
        let _ps = dynamodb_cqrs(client, vec![Arc::new(query)]);
    }
}
