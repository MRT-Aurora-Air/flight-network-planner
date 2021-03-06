use crate::types::*;
use anyhow::{anyhow, Result};
use counter::Counter;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub airline_name: AirlineName,
    pub airline_code: String,
    ignored_airlines: Vec<AirlineName>,
    hubs: Vec<AirportCode>,
    hub_threshold: usize,
    pub range_h2h: Vec<(FlightNumber, FlightNumber)>,
    pub range_n2n: Vec<(FlightNumber, FlightNumber)>,
    pub range_h2n: HashMap<AirportCode, Vec<(FlightNumber, FlightNumber)>>,
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
    #[serde(skip)]
    gates: Vec<Gate>,
}
impl Config {
    pub fn airports(&mut self) -> Result<Vec<AirportCode>> {
        Ok(self
            .gates()?
            .into_iter()
            .map(|g| g.airport)
            .sorted()
            .dedup()
            .collect())
    }
    pub fn hubs(&mut self) -> Result<Vec<AirportCode>> {
        Ok(if !self.hubs.is_empty() {
            self.hubs.clone()
        } else {
            self.gates()?
                .into_iter()
                .map(|g| g.airport)
                .collect::<Counter<_>>()
                .into_iter()
                .filter(|(_, c)| *c >= self.hub_threshold)
                .map(|(a, _)| a)
                .collect::<Vec<_>>()
        })
    }
    pub fn gates(&mut self) -> Result<Vec<Gate>> {
        if self.gates.is_empty() {
            let gates = std::fs::read_to_string(&self.gate_file)?
                .split('\n')
                .filter(|l| !l.is_empty())
                .map(|l| {
                    Some({
                        let params = l.split(' ').collect::<Vec<_>>();
                        Gate {
                            airport: params.get(0)?.trim().to_string(),
                            code: params.get(1)?.trim().to_string(),
                            size: params.get(2)?.trim().to_string()
                        }
                    })
                })
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| anyhow!("Invalid gate file"));
            self.gates = gates?;
        }
        Ok(self.gates.to_owned())
    }
    pub fn ignored_airlines(&self) -> Vec<AirlineName> {
        if self.ignored_airlines.is_empty() {
            vec![self.airline_name.to_owned()]
        } else {
            self.ignored_airlines.to_owned()
        }
    }
}
