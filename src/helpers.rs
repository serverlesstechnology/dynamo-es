use aws_sdk_dynamodb::client::Client;
use aws_sdk_dynamodb::model::{AttributeValue, TransactWriteItem};
use aws_sdk_dynamodb::output::QueryOutput;
use serde_json::Value;

use crate::error::DynamoAggregateError;

pub(crate) async fn load_dynamo_view(
    client: &Client,
    table_name: &str,
    view_id: &str,
) -> Result<QueryOutput, DynamoAggregateError> {
    Ok(client
        .query()
        .table_name(table_name)
        .key_condition_expression("#view_type_id = :view_type_id")
        .expression_attribute_names("#view_type_id", "QueryInstanceId")
        .expression_attribute_values(":view_type_id", AttributeValue::S(String::from(view_id)))
        .send()
        .await?)
}

pub(crate) async fn commit_transactions(
    client: &Client,
    transactions: Vec<TransactWriteItem>,
) -> Result<(), DynamoAggregateError> {
    let transaction_len = transactions.len();
    if transaction_len > 25 {
        return Err(DynamoAggregateError::TransactionListTooLong(
            transaction_len,
        ));
    }
    client
        .transact_write_items()
        .set_transact_items(Some(transactions))
        .send()
        .await?;
    Ok(())
}

pub(crate) fn att_as_value(attribute: Option<&AttributeValue>) -> Value {
    let payload_blob = attribute.unwrap().as_b().unwrap();
    let payload = serde_json::from_slice(payload_blob.clone().into_inner().as_slice()).unwrap();
    payload
}

pub(crate) fn att_as_number(attribute: Option<&AttributeValue>) -> usize {
    attribute.unwrap().as_n().unwrap().parse().unwrap()
}

pub(crate) fn att_as_string(attribute: Option<&AttributeValue>) -> String {
    attribute.unwrap().as_s().unwrap().to_string()
}
