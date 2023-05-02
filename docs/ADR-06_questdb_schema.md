# Architecture Design Record
## ADR-06, QuestDb Schema

https://questdb.io/docs/


```json
{
    "device_id": "weather-4",
    "timestamp": "2023-05-02 10:36:27.721070789 UTC",
    "payload": {
        "air_quality": 5.0,
        "humidity": 81.0,
        "pressure": 31.0,
        "temperature": 1.0
    }
}
```

QuestDb
```sql
CREATE TABLE Device (
  device_id LONG,
  device_uuid UUID,
  device_name STRING,
  device_type SYMBOL INDEX,
  location STRING,
  description STRING,
  installation_date TIMESTAMP
);

CREATE TABLE Sensor (
  sensor_id LONG,
  sensor_uuid UUID,
  sensor_name STRING,
  sensor_type SYMBOL INDEX,
  sensor_description STRING,
  sensor_unit SYMBOL,
  sensor_unit_multiplier DOUBLE
);

CREATE TABLE Datapoint (
  telemetry_id LONG,
  device_id LONG,
  sensor_id LONG,
  timestamp TIMESTAMP,
  value DOUBLE
) timestamp (timestamp) PARTITION BY YEAR;

```