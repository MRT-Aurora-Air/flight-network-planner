use crate::flight_type::FlightType;
use crate::types::{AirportCode, FlightNumber, FlightUtils, GateCode, Size};
use crate::{Config, FlightData};
use anyhow::anyhow;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Flight {
    pub flight_number: FlightNumber,
    pub airport1: (AirportCode, GateCode),
    pub airport2: (AirportCode, GateCode),
    pub size: Size,
}

impl Display for Flight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}): {} {} {} {}",
            self.flight_number,
            self.size,
            self.airport1.0,
            self.airport1.1,
            self.airport2.0,
            self.airport2.1
        )
    }
}

impl FlightUtils for Flight {
    fn score(&self, config: &mut Config, flight_data: &FlightData) -> anyhow::Result<i8> {
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
    fn get_flight_type(
        &self,
        config: &mut Config,
        flight_data: &FlightData,
    ) -> anyhow::Result<FlightType> {
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
