# Architecture Design Record

## ADR-12, Connection Flow

1. Device Twin get created on the web app.

- this gives the etag and device id
- we pass this info to oxi and dizer at startup time (config file or compiled)

2. Device get online and his connected to the network

- device sends heartbeat every 5 mins
- device subscribes to desired properties
- device ask for desired properties
- $version of the reported properties always increment, discard lower.

<https://github.com/surrealdb/surrealdb/blob/main/lib/examples/query/main.rs>

