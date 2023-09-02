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
│     ├─ meta     
│     ├─ desired
├─ create/          // must specify a device id
│  ├─ device
├─ delete/          // must specify a device id
│  ├─ device
```
