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
  --item file://db/upcast_test_entry.json \
  --endpoint-url http://localhost:8000

aws dynamodb get-item \
  --table-name Events \
  --key '{"AggregateTypeAndId": {"S": "Customer:previous_event_in_need_of_upcast"}, "AggregateIdSequence": {"N": "1"}}' \
  --endpoint-url http://localhost:8000