use crate::config::Config;
use crate::FlightData;
use anyhow::{anyhow, Result};
use std::fmt::Display;

pub type AirlineName = String;
pub type AirportCode = String;
pub type GateCode = String;
pub type FlightNumber = u16;
pub type Size = String;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub enum FlightType {
    NonExistingH2H,
    ExistingH2H,
    NonExistingH2N,
    NonExistingN2N,
    ExistingH2N,
    ExistingN2N,
}
impl Display for FlightType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlightType::NonExistingH2H => write!(f, "H2Hn"),
            FlightType::ExistingH2H => write!(f, "H2He"),
            FlightType::NonExistingH2N => write!(f, "H2Nn"),
            FlightType::NonExistingN2N => write!(f, "N2Nn"),
            FlightType::ExistingH2N => write!(f, "H2Ne"),
            FlightType::ExistingN2N => write!(f, "N2Ne"),
        }
    }
}
impl FlightType {
    pub fn score(&self) -> u8 {
        match self {
            FlightType::NonExistingH2H => 6,
            FlightType::ExistingH2H => 5,
            FlightType::NonExistingH2N => 3,
            FlightType::NonExistingN2N => 2,
            FlightType::ExistingH2N => 1,
            FlightType::ExistingN2N => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Gate {
    pub airport: AirportCode,
    pub code: GateCode,
    pub size: Size,
}
impl Display for Gate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} ({})", self.airport, self.code, self.size)
    }
}

pub trait FlightUtils {
    fn score(&self, config: &mut Config, flight_data: &FlightData) -> Result<i8>;
    fn get_flight_type(&self, config: &mut Config, flight_data: &FlightData) -> Result<FlightType>;
}
impl FlightUtils for (&AirportCode, &AirportCode) {
    fn score(&self, config: &mut Config, flight_data: &FlightData) -> Result<i8> {
        let mut s = 0i8;

        s -= flight_data.num_flights(self.0, self.1) as i8 - 1;
        if s == 1 {
            s += 1;
        }

        s += self.get_flight_type(config, flight_data)?.score() as i8;

        if config
            .preferred_between
            .iter()
            .any(|fs| fs.contains(self.0) && fs.contains(self.1))
        {
            s += 10;
        }
        if let Some(dests) = config.preferred_to.get(self.0) {
            if dests.contains(self.1) {
                s += 10;
            }
        }
        if let Some(dests) = config.preferred_to.get(self.1) {
            if dests.contains(self.0) {
                s += 10;
            }
        }

        Ok(s)
    }
    fn get_flight_type(&self, config: &mut Config, flight_data: &FlightData) -> Result<FlightType> {
        Ok(if config.hubs()?.contains(self.0) {
            if config.hubs()?.contains(self.1) {
                if flight_data.num_flights(self.0, self.1) > 0 {
                    FlightType::ExistingH2H
                } else {
                    FlightType::NonExistingH2H
                }
            } else if flight_data.num_flights(self.0, self.1) > 0 {
                FlightType::ExistingH2N
            } else {
                FlightType::NonExistingH2N
            }
        } else if config.hubs()?.contains(self.1) {
            if flight_data.num_flights(self.0, self.1) > 0 {
                FlightType::ExistingH2N
            } else {
                FlightType::NonExistingH2N
            }
        } else if flight_data.num_flights(self.0, self.1) > 0 {
            FlightType::ExistingN2N
        } else {
            FlightType::NonExistingN2N
        })
    }
}
impl FlightUtils for (&Gate, &Gate) {
    fn score(&self, config: &mut Config, flight_data: &FlightData) -> Result<i8> {
        let mut s = (&self.0.airport, &self.1.airport).score(config, flight_data)?;
        if &*self.0.size != "S" {
            s += 2;
        }
        Ok(s)
    }
    fn get_flight_type(&self, config: &mut Config, flight_data: &FlightData) -> Result<FlightType> {
        (&self.0.airport, &self.1.airport).get_flight_type(config, flight_data)
    }
}

#[derive(Debug, Clone)]
pub struct Flight {
    pub flight_number: FlightNumber,
    pub airport1: (AirportCode, GateCode),
    pub airport2: (AirportCode, GateCode),
    pub size: Size
}
impl Display for Flight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}): {} {} {} {}",
            self.flight_number, self.size, self.airport1.0, self.airport1.1, self.airport2.0, self.airport2.1
        )
    }
}
impl FlightUtils for Flight {
    fn score(&self, config: &mut Config, flight_data: &FlightData) -> Result<i8> {
        (
            &config
                .gates()?
                .into_iter()
                .find(|g| g.code == self.airport1.0)
                .ok_or_else(|| anyhow!("Gate not found"))?,
            &config
                .gates()?
                .into_iter()
                .find(|g| g.code == self.airport2.0)
                .ok_or_else(|| anyhow!("Gate not found"))?,
        )
            .score(config, flight_data)
    }
    fn get_flight_type(&self, config: &mut Config, flight_data: &FlightData) -> Result<FlightType> {
        (
            &config
                .gates()?
                .into_iter()
                .find(|g| g.code == self.airport1.0)
                .ok_or_else(|| anyhow!("Gate not found"))?,
            &config
                .gates()?
                .into_iter()
                .find(|g| g.code == self.airport2.0)
                .ok_or_else(|| anyhow!("Gate not found"))?,
        )
            .get_flight_type(config, flight_data)
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
