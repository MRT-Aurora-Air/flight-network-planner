use crate::types::*;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Config {
    pub airline_name: AirlineName,
    pub airline_code: String,
    pub ignored_airlines: Vec<AirlineName>,
    hubs: Vec<AirportCode>,
    hub_threshold: u8,
    pub range_h2h: Vec<(AirportCode, AirportCode)>,
    pub range_n2n: Vec<(AirportCode, AirportCode)>,
    pub range_h2n: HashMap<AirportCode, Vec<(AirportCode, AirportCode)>>,
    pub both_dir_same_num: bool,
    pub gate_file: PathBuf,
    pub hard_max_hub: u8,
    pub hard_max_nonhub: u8,
    pub max_h2h: u8,
    pub max_h2n: u8,
    pub max_n2n: u8,
    pub restricted_between: Vec<Vec<AirportCode>>,
    pub restricted_to: HashMap<AirportCode, Vec<AirportCode>>,
    pub preferred_between: Vec<Vec<AirportCode>>,
    pub preferred_to: HashMap<AirportCode, Vec<AirportCode>>,
    pub gate_allowed_dests: HashMap<AirportCode, HashMap<GateCode, Vec<AirportCode>>>,
    pub gate_denied_dests: HashMap<AirportCode, HashMap<GateCode, Vec<AirportCode>>>,
}
impl Config {
    pub fn hubs(&self) -> Vec<AirportCode> {
        if self.hubs.is_empty() {
            self.hubs.clone()
        } else {
            vec![]
        }
    }
}
