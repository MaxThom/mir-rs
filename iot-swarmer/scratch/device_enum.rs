//
//
//
//
//
//
//
//se core::fmt::Error;
//
//se std::fmt;
//
//
//
///#[derive(Debug, Clone)]
//
//ub struct Swarm {
//
//   pub devices: Vec<LiveDevice>,
//
//
//
//
//
//mpl Swarm {
//
//   pub fn new() -> Result<Self, Error> {
//
//       Ok(Self {
//
//           devices: Vec::new(),
//
//       })
//
//   }
//
//
//
//   pub fn add_device(&mut self, device: LiveDevice) {
//
//       self.devices.push(device);
//
//   }
//
//
//
//
//
///#[derive(Debug, Clone)]
//
//ub struct LiveDevice {
//
//   pub name: String,
//
//   pub sensors: Vec<LiveSensor>,
//
//
//
//
//
//mpl LiveDevice {
//
//   pub fn new(name: String) -> Result<Self, Error> {
//
//       Ok(Self {
//
//           name,
//
//           sensors: Vec::new(),
//
//       })
//
//   }
//
//
//
//   pub fn add_sensor(&mut self, sensor: LiveSensor) -> &mut Self {
//
//       self.sensors.push(sensor);
//
//       self
//
//   }
//
//
//
//
//
//ub struct LiveSensor {
//
//   pub name: String,
//
//   pub hysteresis: f32,
//
//   pub telemetry: TelemetryType,
//
//   previous_datapoint: f32,
//
//   rate: f32,
//
//
//
//
//
//mpl LiveSensor {
//
//   pub fn new(name: String, hysteresis: f32, telemetry: TelemetryType) -> Self {
//
//       Self {
//
//           name,
//
//           hysteresis,
//
//           telemetry,
//
//           previous_datapoint: telemetry.get_initial_datapoint(),
//
//           rate: telemetry.get_rate(),
//
//       }
//
//   }
//
//   fn next_datapoint(&self) -> Option<f32> {
//
//       let mut x = self.rate;
//
//       self.previous_datapoint = self.telemetry.next_datapoint(self.previous_datapoint(), &mut x);
//
//       
//
//       return Some(self.previous_datapoint)
//
//   }
//
//   fn previous_datapoint(&self) -> f32 {
//
//       self.previous_datapoint
//
//   }
//
//
//
//
//
//
//
//mpl fmt::Debug for LiveSensor {
//
//   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//
//       f.debug_struct("Sensor")
//
//        .field("name", &self.name)
//
//        .field("hysteresis", &self.hysteresis)
//
//        .finish()
//
//   }
//
//
//
//
//
//ub enum TelemetryType {
//
//   Linear(f32),
//
//   Pyramid(f32, f32, f32),
//
//   Wave(f32, f32, f32),
//
//   //Real,
//
//
//
//
//
//mpl TelemetryType {
//
//   fn next_datapoint(&self, previous_datapoint: f32, p_rate: &mut f32) -> f32 {
//
//       match self {
//
//           TelemetryType::Linear(constant) => {
//
//               *constant
//
//           },
//
//           TelemetryType::Pyramid(rate, min, max) => {
//
//               let value = previous_datapoint + *rate;
//
//               if value > *max {
//
//                   *p_rate = *p_rate * -1.0;
//
//               } else if value < *min {
//
//                   *p_rate = *p_rate * -1.0;
//
//               }
//
//               value
//
//           },
//
//           TelemetryType::Wave(rate, min, max) => {
//
//               let mut value = previous_datapoint + *rate;
//
//               if value > *max {
//
//                   value = *min;
//
//               }
//
//               value
//
//           }
//
//       }
//
//   }
//
//   fn get_initial_datapoint(&self) -> f32 {
//
//       match self {
//
//           TelemetryType::Linear(constant) => {
//
//               *constant
//
//           },
//
//           TelemetryType::Pyramid(rate, min, max) => {
//
//               *min
//
//           },
//
//           TelemetryType::Wave(rate, min, max) => {
//
//               *min
//
//           }
//
//       }
//
//   }
//
//   fn get_rate(&self) -> f32 {
//
//       match self {
//
//           TelemetryType::Linear(constant) => {
//
//               *constant
//
//           },
//
//           TelemetryType::Pyramid(rate, min, max) => {
//
//               *rate
//
//           },
//
//           TelemetryType::Wave(rate, min, max) => {
//
//               *rate
//
//           }
//
//       }
//
//   }
//
//
//
//
//
//
//
//
//
//