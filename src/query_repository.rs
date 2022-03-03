use std::marker::PhantomData;

use async_trait::async_trait;
use aws_sdk_dynamodb::model::{AttributeValue, Put, TransactWriteItem};
use aws_sdk_dynamodb::Blob;
use cqrs_es::persist::{PersistenceError, QueryContext, ViewRepository};
use cqrs_es::{Aggregate, View};

use crate::helpers::{att_as_number, att_as_value, commit_transactions, load_dynamo_view};

/// A DynamoDb backed query repository for use in backing a `GenericQuery`.
pub struct DynamoViewRepository<V, A> {
    _phantom: PhantomData<(V, A)>,
    query_name: String,
    client: aws_sdk_dynamodb::client::Client,
}

impl<V, A> DynamoViewRepository<V, A>
where
    V: View<A>,
    A: Aggregate,
{
    /// Creates a new `DynamoViewRepository` that will store serialized views in a DynamoDb table named
    /// identically to the `query_name` value provided. This table should be created by the user
    /// before using this query repository (see `Makefile` for `create-table` command
    /// line example).
    pub fn new(query_name: &str, client: aws_sdk_dynamodb::client::Client) -> Self {
        Self {
            _phantom: Default::default(),
            query_name: query_name.to_string(),
            client,
        }
    }
}

#[async_trait]
impl<V, A> ViewRepository<V, A> for DynamoViewRepository<V, A>
where
    V: View<A>,
    A: Aggregate,
{
    async fn load(
        &self,
        query_instance_id: &str,
    ) -> Result<Option<(V, QueryContext)>, PersistenceError> {
        let query_result =
            load_dynamo_view(&self.client, &self.query_name, query_instance_id).await?;
        let query_items = match query_result.items {
            None => return Ok(None),
            Some(items) => items,
        };
        let query_item = query_items.get(0).expect("only one query item");
        let version = att_as_number(query_item.get("ViewVersion"));
        let payload = att_as_value(query_item.get("Payload"));
        let view: V = serde_json::from_value(payload)?;
        let context = QueryContext::new(query_instance_id.to_string(), version as i64);
        Ok(Some((view, context)))
    }

    async fn update_view(&self, view: V, context: QueryContext) -> Result<(), PersistenceError> {
        let query_instance_id = AttributeValue::S(String::from(&context.view_instance_id));
        let expected_view_version = AttributeValue::N(context.version.to_string());
        let view_version = AttributeValue::N((context.version + 1).to_string());
        let payload_blob = serde_json::to_vec(&view).unwrap();
        let payload = AttributeValue::B(Blob::new(payload_blob));
        let transaction = TransactWriteItem::builder()
            .put(Put::builder()
                .table_name(&self.query_name)
                .item("QueryInstanceId", query_instance_id)
                .item("ViewVersion", view_version)
                .item("Payload", payload)
                .condition_expression("attribute_not_exists(ViewVersion) OR (ViewVersion  = :expected_view_version)")
                .expression_attribute_values(":expected_view_version", expected_view_version)
                .build())
            .build();
        commit_transactions(&self.client, vec![transaction]).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use cqrs_es::persist::{QueryContext, ViewRepository};

    use crate::testing::tests::{
        test_dynamodb_client, Created, TestAggregate, TestEvent, TestView,
    };
    use crate::DynamoViewRepository;

    #[tokio::test]
    async fn test_valid_view_repository() {
        let repo = DynamoViewRepository::<TestView, TestAggregate>::new(
            "TestQuery",
            test_dynamodb_client().await,
        );
        let test_view_id = uuid::Uuid::new_v4().to_string();

        let view = TestView {
            events: vec![TestEvent::Created(Created {
                id: "just a test event for this view".to_string(),
            })],
        };
        repo.update_view(view.clone(), QueryContext::new(test_view_id.to_string(), 0))
            .await
            .unwrap();
        let (found, context) = repo.load(&test_view_id).await.unwrap().unwrap();
        assert_eq!(found, view);

        let updated_view = TestView {
            events: vec![TestEvent::Created(Created {
                id: "a totally different view".to_string(),
            })],
        };
        repo.update_view(updated_view.clone(), context)
            .await
            .unwrap();
        let found_option = repo.load(&test_view_id).await.unwrap();
        let found = found_option.unwrap().0;

        assert_eq!(found, updated_view);
    }
}
