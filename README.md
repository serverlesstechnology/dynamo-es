# dynamo-es

> A DynamoDB implementation of the `PersistedEventRepository` trait in cqrs-es.
### DynamoDb caveats
AWS' DynamoDb is fast, flexible and highly available, but it does 
[make some limitations](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ServiceQuotas.html)
that must be considered in the design of your application.

- Maximum limit of 25 operations in any transaction - Events are inserted in a single transaction, this means that a
command can not produce more than 25 events using Event store, or 24 events if using Snapshot or Aggregate store.
This is rare but if a command might produce more than this limit a different backing database should be used. 
- Item size limit of 400 KB - An event should never reach this size, but it is possible that a serialized aggregate might.
If this is possible for your use case beware of using Aggregate or Snapshot stores.
- Maximum request size of 1 MB - This may have the same ramifications as the above for Aggregate or Snapshot stores.
Additionally, large numbers of events may reach this threshold. 
To prevent an error while loading or replaying events, 
[set the streaming channel size](https://docs.rs/dynamo-es/0.4.2/dynamo_es/struct.DynamoEventRepository.html#method.with_streaming_channel_size)
to a number that ensures you won't exceed this threshold.


### Testing

Requires access to DynamoDb with existing tables. This can be created locally using the included 
`docker-compose.yml` file with CLI configuration of test tables included in the `Makefile`.

To prepare a local test environment (requires a local installation of 
[Docker](https://www.docker.com/products/docker-desktop) and 
[AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html)):
- `docker-compose up -d`
- `make configure`

Note that this crate used the [AWS DynamoDb Rust SDK](https://aws.amazon.com/sdk-for-rust/), which is currently in 
Developer Preview. This means that any bugs will be addressed but the underlying interfaces may still be changed 
resulting in significant changes within this crate. See the 
[AWS SDK public roadmap for more information](https://github.com/awslabs/aws-sdk-rust/projects/1).

It is recommended that tables are configured to allow only transactions.
See:
https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/transaction-apis-iam.html

---

Things that could be helpful:
- [User guide](https://doc.rust-cqrs.org) along with an introduction to CQRS and event sourcing.
- [Demo application](https://github.com/serverlesstechnology/cqrs-demo) using the warp http server.
- [Change log](https://github.com/serverlesstechnology/cqrs/blob/master/change_log.md)

[![Crates.io](https://img.shields.io/crates/v/dynamo-es)](https://crates.io/crates/dynamo-es)
[![docs](https://img.shields.io/badge/API-docs-blue.svg)](https://docs.rs/dynamo-es)
![build status](https://codebuild.us-west-2.amazonaws.com/badges?uuid=eyJlbmNyeXB0ZWREYXRhIjoiVVUyR0tRbTZmejFBYURoTHdpR3FnSUFqKzFVZE9JNW5haDZhcUFlY2xtREhtaVVJMWsxcWZOeC8zSUR0UWhpaWZMa0ZQSHlEYjg0N2FoU2lwV1FsTXFRPSIsIml2UGFyYW1ldGVyU3BlYyI6IldjUVMzVEpKN1V3aWxXWGUiLCJtYXRlcmlhbFNldFNlcmlhbCI6MX0%3D&branch=master)
