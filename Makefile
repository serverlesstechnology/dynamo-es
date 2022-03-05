
configure:
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
		--table-name TestQuery \
        --key-schema \
        	AttributeName=ViewId,KeyType=HASH \
		--attribute-definitions \
        	AttributeName=ViewId,AttributeType=S \
		--billing-mode PAY_PER_REQUEST \
		--endpoint-url http://localhost:8000

test: configure
	cargo test

doc:
	cargo doc --lib --no-deps
