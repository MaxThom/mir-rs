# Architecture Design Record
## ADR-07, Device Twin

https://learn.microsoft.com/en-us/azure/iot-hub/iot-hub-devguide-device-twins

## Metadata

```json
"deviceId": "devA", // device id
"etag": "AAAAAAAAAAc=", // model id
"status": "enabled", // enabled, disabled
"statusReason": "provisioned", // provisioned, registered, blocked, unblocked
"statusUpdateTime": "0001-01-01T00:00:00", // timestamp UTC
"connectionState": "connected", // connected, disconnected
"lastActivityTime": "2015-02-30T16:24:48.789Z", // timestamp UTC
"cloudToDeviceMessageCount": 0, // count
"version": 2, // might get useful for versioning
```

statusReason:

- provisioned: his twin was created in the database
- registered: the device has connected to the hub

## Tags

- Set of tags that can be used to filter devices.
- Tags are key-value pairs.
- Tags are stored in the device twin.
- Tags can be updated by the back-end application.
- Tags cannot be read by the deivce application.
- Tags can be used to filter devices for bulk operations.
- Tags can be used to filter devices for routing messages.
- Tags can be used to filter devices for monitoring.
- Tags can be used to filter devices for device twin operations.
- Tags can be used to filter devices for jobs.
- Tags can be used to filter devices for direct methods.
- Tags can be used to filter devices for device twin queries.
- Tags can be used to filter devices for device twin jobs.
- Tags can be used to filter devices for device twin queries.
- Tags can be used to filter devices for device twin jobs.
- Tags can be used to filter devices for device twin queries.
- Tags can be used to filter devices for device twin jobs.
- Tags can be used to filter devices for device twin queries.
- Tags can be used to filter devices for device twin jobs.
- Tags can be used to filter devices for device twin queries.
- Tags can be used to filter devices for device twin jobs.
- Tags can be used to filter devices for device twin queries.
- Tags can be used to filter devices for device twin jobs.
- Tags can be used to filter devices for device twin queries.
- Tags can be used to filter devices for device twin jobs.
- Tags can be used to filter devices for device twin queries.
- Tags can be used to filter devices for device twin jobs.


## Desired Properties

- Desired properties are set by the back-end application.
- Desired properties are read by the device app.
- Desired properties are used when we are interested in the last value of a property. eg: last known location.

## Reported Properties

- Reported properties are set by the device app.
- Reported properties are read by the back-end application.

```json
{
    "deviceId": "devA",
    "etag": "AAAAAAAAAAc=",
    "status": "enabled",
    "statusReason": "provisioned",
    "statusUpdateTime": "0001-01-01T00:00:00",
    "connectionState": "connected",
    "lastActivityTime": "2015-02-30T16:24:48.789Z",
    "cloudToDeviceMessageCount": 0,
    "version": 2,
    "tags": { 
        "deploymentLocation": {
            "building": "43",
            "floor": "1"
        }
    },
    "properties": {
        "desired": {
            "telemetryConfig": {
                "sendFrequency": "5m"
            },
            "$metadata" : {...}, // timestamp of each field with last update
            "$version": 1
        },
        "reported": {
            "telemetryConfig": {
                "sendFrequency": "5m",
                "status": "success"
            },
            "batteryLevel": 55,
            "$metadata" : {...}, // timestamp of each field with last update
            "$version": 4
        }
    }
}
```