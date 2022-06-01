#!/bin/bash
aws dynamodb create-table \
  --table-name Events \
      --key-schema \
        AttributeName=AggregateTypeAndId,KeyType=HASH \
        AttributeName=AggregateIdSequence,KeyType=RANGE \
  --attribute-definitions \
        AttributeName=AggregateTypeAndId,AttributeType=S \
        AttributeName=AggregateIdSequence,AttributeType=N \
  --billing-mode PAY_PER_REQUEST \
  --endpoint-url http://localhost:8000

aws dynamodb create-table \
  --table-name Snapshots \
      --key-schema \
        AttributeName=AggregateTypeAndId,KeyType=HASH \
  --attribute-definitions \
        AttributeName=AggregateTypeAndId,AttributeType=S \
  --billing-mode PAY_PER_REQUEST \
  --endpoint-url http://localhost:8000

aws dynamodb create-table \
  --table-name TestViewTable \
      --key-schema \
        AttributeName=ViewId,KeyType=HASH \
  --attribute-definitions \
        AttributeName=ViewId,AttributeType=S \
  --billing-mode PAY_PER_REQUEST \
  --endpoint-url http://localhost:8000

aws dynamodb put-item \
  --table-name Events \
      --item '{
                  "AggregateTypeAndId": {"S": "Customer:previous_event_in_need_of_upcast"},
                  "AggregateIdSequence": {"N": "1"},
                  "AggregateType": {"S": "Customer"},
                  "AggregateId": {"S": "previous_event_in_need_of_upcast"},
                  "EventVersion": {"S": "1.0"},
                  "EventType": {"S": "NameAdded"},
                  "Payload": {"B": "eyJOYW1lQWRkZWQiOiB7fX0="},
                  "Metadata": {"B": "e30="}
              }' \
      --endpoint-url http://localhost:8000

