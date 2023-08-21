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
  - [ ] update device_twin meta section with heartbeat

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
      - [ ] client side, set up callback and await message from queue
      - [ ] server side
- [ ] retrieve desired properties
  - connect to desired properties queue. [See ADR-11], --> no need for persistent queue if device disconect
  - get desired properties using rest api
  - both type of getting properties are receive in the same callback
- [ ] update reported properties

## Cockpit

- [ ] start
