use crate::config::Config;
use crate::FlightData;
use anyhow::Result;

pub type AirlineName = String;
pub type AirportCode = String;
pub type GateCode = String;
pub type FlightNumber = u16;
pub type Size = String;

pub enum FlightType {
    ExistingH2H,
    ExistingH2N,
    ExistingN2N,
    NonExistingH2H,
    NonExistingH2N,
    NonExistingN2N,
}

#[derive(Debug, Clone)]
pub struct Gate {
    pub airport: AirportCode,
    pub code: GateCode,
    pub size: Size,
    pub destinations: Vec<AirportCode>,
    pub score: i8,
}

#[derive(Debug, Clone)]
pub struct Flight {
    pub airline: AirlineName,
    pub flight_number: FlightNumber,
    pub airport1: (AirportCode, GateCode),
    pub airport2: (AirportCode, GateCode),
}
impl Flight {
    pub fn score() -> i8 {
        0
    }
    pub fn get_flight_type(
        &self,
        config: &mut Config,
        flight_data: &FlightData,
    ) -> Result<FlightType> {
        Ok(if config.hubs()?.contains(&self.airport1.0) {
            if config.hubs()?.contains(&self.airport2.0) {
                if flight_data.num_flights(&self.airport1.0, &self.airport2.0) > 0 {
                    FlightType::ExistingH2H
                } else {
                    FlightType::NonExistingH2H
                }
            } else if flight_data.num_flights(&self.airport1.0, &self.airport2.0) > 0 {
                FlightType::ExistingH2N
            } else {
                FlightType::NonExistingH2N
            }
        } else if config.hubs()?.contains(&self.airport2.0) {
            if flight_data.num_flights(&self.airport1.0, &self.airport2.0) > 0 {
                FlightType::ExistingH2N
            } else {
                FlightType::NonExistingH2N
            }
        } else if flight_data.num_flights(&self.airport1.0, &self.airport2.0) > 0 {
            FlightType::ExistingN2N
        } else {
            FlightType::NonExistingN2N
        })
    }
}

pub struct FlightNumberGenerator(Box<dyn Iterator<Item = FlightNumber>>);
impl FlightNumberGenerator {
    pub fn new(numbers: Vec<(FlightNumber, FlightNumber)>) -> Self {
        Self(Box::new(numbers.into_iter().flat_map(|(a, b)| a..=b)))
    }
    pub fn next(&mut self) -> Option<FlightNumber> {
        self.0.next()
    }
}
