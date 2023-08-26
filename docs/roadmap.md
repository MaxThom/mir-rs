# Roadmap


## Redox

- [x] create a device twin from api
- [x] able to retrieve device twin or parts of it via api
- [x] able to update tags for twin
- [x] able to update desired properties for twin
  - [x] send a message to the device with updated desired properties
- [ ] read reported properties messages
  - [ ] update reported properties of device twin in db
- [x] read meta messages
  - [x] update device_twin meta section with heartbeat

## Swarm

- [x] basic functionality to have telemetry at will
- [] recode with new dizer library
- [] add device twin support

## Oxi

- [ ] start

## Dizer

- [x] create builder pattern for dizer
- [x] config, logging, cli
- [x] send telemetry
- [x] heartbeat + metadata
- [ ] initial pulling of desired properties
  - [ ] using rpc
    - [x] client side, send message
    - [x] client side, set up callback and await message from queue
    - [x] server side
    - [ ] remove rpcclient and set a queue listen in AMQP
- [x] retrieve desired properties
  - [x] connect to desired properties queue. [See ADR-11], --> no need for persistent queue if device disconect
  - [x] in redox, add new properties update in rest endpoint to device queue
- [ ] update reported properties

## Cockpit

- [ ] start
