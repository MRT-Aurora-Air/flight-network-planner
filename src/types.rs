use crate::config::Config;
use crate::flight_type::FlightType;
use crate::gate::Gate;
use crate::FlightData;
use anyhow::Result;

pub type AirlineName = String;
pub type AirportCode = String;
pub type GateCode = String;
pub type FlightNumber = u16;
pub type Size = String;

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

        s += self.get_flight_type(config, flight_data)?.score();

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
