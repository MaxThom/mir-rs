# Architecture Design Record
## ADR-03, Payload

```json
{
    "device_id": "xxxx",
    "timestamp": "utc_date",
    "payload": {
        "temperature": "xx",
        ...key-value pair...
    }
}
```

In the framework, 
send_telemetry every x minutes, timeout y minutes.

for each telemetry point, pass a callback function that return the new value and a Hysteresis field (+- 0.5).

The framework iterate on the callbacks and if the payload is new or timeout reached, send messages.