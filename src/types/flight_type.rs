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
            Self::NonExistingH2H => write!(f, "H2Hn"),
            Self::ExistingH2H => write!(f, "H2He"),
            Self::NonExistingH2N => write!(f, "H2Nn"),
            Self::NonExistingN2N => write!(f, "N2Nn"),
            Self::ExistingH2N => write!(f, "H2Ne"),
            Self::ExistingN2N => write!(f, "N2Ne"),
        }
    }
}

impl FlightType {
    pub fn score(&self) -> i8 {
        match self {
            Self::NonExistingH2H => 6,
            Self::ExistingH2H => 5,
            Self::NonExistingH2N => 3,
            Self::NonExistingN2N => 2,
            Self::ExistingH2N => 1,
            Self::ExistingN2N => -1,
        }
    }
}
