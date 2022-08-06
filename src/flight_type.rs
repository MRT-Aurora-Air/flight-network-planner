use std::fmt::Display;

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
