# Architecture Design Record

## ADR-13, CLI

```csharp
mir/
├─ list/
│  ├─ devices/      // can specify a device id
│     ├─ all        // not a command, its devices
│     ├─ meta     
│     ├─ desired
│     ├─ reported
├─ update/
│  ├─ device/       // must specify a device id
│     ├─ tag     
│     ├─ desired
├─ create/          // must specify a device id
│  ├─ device
├─ delete/          // must specify a device id
│  ├─ device
├─ stream/          // stream data. add filter based on queue type.
│  ├─ telemetry
├─ listen/          // listen to all queue at the same time. add filter based on queue type.
│  ├─ hearthbeat/
│  ├─ desired/
│  ├─ reported/
│  ├─ telemetry/
│  ├─ .../
```

### Derive vs Builder

Both pattern seems interesting, but builder seems more flexible.

### File strcture

Separate file similar to go cobra. Each command or subcommand has a file
