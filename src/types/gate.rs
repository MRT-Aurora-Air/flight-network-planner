use std::fmt::Display;

use crate::types::{AirportCode, GateCode, Size};

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
