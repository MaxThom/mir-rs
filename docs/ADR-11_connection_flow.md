# Architecture Design Record
## ADR-11, Connection Flow

1. Device Twin get created on the web app.
  - this gives the etag and device id
  - we pass this info to oxi and dizer at startup time (config file or compiled)
2. Device get online and his connected to the network
  - device sends heartbeat every 5 mins
  - device subscribes to desired properties
  - device ask for desired properties
  - $version of the reported properties always increment, discard lower.

// https://github.com/surrealdb/surrealdb/blob/main/lib/examples/query/main.rs

TODO - Redox
- [x] create a device twin from api
- [x] able to retrieve device twin or parts of it via api
- [ ] able to update tags for twin
- [ ] able to update desired properties for twin
  - [ ] send a message to the device with updated desired properties
- [ ] read reported properties messages
  - [ ] update reported properties of device twin in db
- [ ] read meta messages
  - [ ] update device_twin meta section with heartbeat

TODO - Swarm
- [ ] add device twin support

TODO - Oxi
- [ ] start

TODO - Dizer
- [ ] start

TODO - Cockpit
- [ ] start