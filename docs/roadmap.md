# Roadmap


## Redox

- [x] create a device twin from api
- [x] able to retrieve device twin or parts of it via api
- [x] able to update tags for twin
- [x] able to update desired properties for twin
  - [x] send a message to the device with updated desired properties
- [x] read reported properties messages
  - [x] update reported properties of device twin in db
- [x] read meta messages
  - [x] update device_twin meta section with heartbeat
- [ ] add support to return many device ids in one call

## Swarm

- [x] basic functionality to have telemetry at will
- [] recode with new dizer library
- [] add device twin support

## Dizer

- [ ] start

## Oxi

- [x] create builder pattern for dizer
- [x] config, logging, cli
- [x] send telemetry
- [x] send general message
  - [x] with type json serialization
  - [x] your own str
- [x] heartbeat + metadata
- [x] initial pulling of desired properties
  - [x] using rpc
    - [x] client side, send message
    - [x] client side, set up callback and await message from queue
    - [x] server side
    - [x] remove rpcclient and set a queue listen in AMQP
- [x] retrieve desired properties
  - [x] connect to desired properties queue. [See ADR-11], --> no need for persistent queue if device disconect
  - [x] in redox, add new properties update in rest endpoint to device queue
- [x] update reported properties
  - [x] send properties to redox
  - [x] move handler to dizer and not shipyard
  - [ ] add future as callback as well of of FnMut
  - [x] add multiple hanldler
- [ ] caching


## Cli

- [x] list
  - [x] devices
    - [x] all
    - [x] meta
    - [x] desired
    - [x] reported
- [x] update
  - [x] device
    - [x] tag
    - [x] desired
- [x] create
  - [x] device
- [x] delete
  - [x] device



## Cockpit

- [ ] start
