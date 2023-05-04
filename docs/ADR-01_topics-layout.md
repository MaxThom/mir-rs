# Architecture Design Record
## ADR-01, RabbitMQ

RabbitMQ will be used to move telemetry around.

It will be configured as topics with the following pattern.

<device_type>.<telemetry_type>.<version_type>

device_type:

- temperature
- esp32
- weather station

telemetry_type:

- telemetry
- metrics
- logs
- commands
- twin
- registration

version_type:

- format
- major.minor