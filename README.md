# dynamo-es

> A DynamoDB implementation of the `EventStore` trait in cqrs-es.

Requires access to DynamoDb with existing tables. This can be created locally using the included 
`docker-compose.yml` file with CLI configuration of test tables included in the `Makefile`. 

To prepare a local test environment (requires a local installation of 
[Docker](https://www.docker.com/products/docker-desktop) and 
[AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html)):
- `docker-compose up -d`
- `make configure`

Note that this crate used the [AWS DynamoDb Rust SDK](https://aws.amazon.com/sdk-for-rust/), which is currently in 
Developer Preview. This means that bugs will be addressed but the underlying interfaces may still be changed 
resulting in significant changes within this crate. See the 
[AWS SDK public roadmap for more information](https://github.com/awslabs/aws-sdk-rust/projects/1).

It is recommended that tables
See:
https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/transaction-apis-iam.html

---

Things that could be helpful:
- [User guide](https://doc.rust-cqrs.org) along with an introduction to CQRS and event sourcing.
- [Demo application](https://github.com/serverlesstechnology/cqrs-demo) using the warp http server.
- [Change log](https://github.com/serverlesstechnology/cqrs/blob/master/change_log.md)

[![Crates.io](https://img.shields.io/crates/v/dynamo-es)](https://crates.io/crates/dynamo-es)
[![docs](https://img.shields.io/badge/API-docs-blue.svg)](https://docs.rs/dynamo-es)