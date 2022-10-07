use std::fmt::Display;

use crate::types::{AirportCode, GateCode, Size};
use serde::{Serialize, Deserialize};

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


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PartialGate {
    pub code: GateCode,
    pub size: Size
}