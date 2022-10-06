use crate::types::{AirportCode, FlightNumber, GateCode, Size};
use std::fmt::Display;
use crate::types::flight_type::FlightType;

#[derive(Debug, Clone)]
pub struct Flight {
    pub flight_number: FlightNumber,
    pub airport1: (AirportCode, GateCode),
    pub airport2: (AirportCode, GateCode),
    pub size: Size,
    pub score: i8,
    pub flight_type: FlightType,
}

impl Display for Flight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}): {} {} {} {} ({}, {})",
            self.flight_number,
            self.size,
            self.airport1.0,
            self.airport1.1,
            self.airport2.0,
            self.airport2.1,
            self.score,
            self.flight_type
        )
    }
}
