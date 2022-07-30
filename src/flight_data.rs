use crate::config::Config;
use crate::types::*;
use anyhow::{anyhow, Result};
use itertools::Itertools;
use log::{debug, info, warn};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

const PLANE_DATA_URL: &str = "https://sheets.googleapis.com/v4/spreadsheets/1wzvmXHQZ7ee7roIvIrJhkP6oCegnB8-nefWpd8ckqps/values/Airline+Class+Distribution?majorDimension=COLUMNS&key=AIzaSyCCRZqIOAVfwBNUofWbrkz0q5z4FUaCUyE";

#[derive(Debug)]
pub struct FlightDataFlight {
    pub airline: AirlineName,
    pub flight_number: String,
    pub airports: Vec<AirportCode>,
}

#[derive(Debug)]
pub struct FlightData {
    pub flights: Vec<FlightDataFlight>,
    pub airport_codes: Vec<AirportCode>,
    pub timestamp: u64,
}
impl FlightData {
    pub fn from_sheets() -> Result<Self> {
        let raw = reqwest::blocking::get(PLANE_DATA_URL)?.json::<Value>()?["values"]
            .as_array()
            .ok_or_else(|| anyhow!("Incorrect format"))?
            .iter()
            .map(|r| {
                r.as_array()
                    .ok_or_else(|| anyhow!("Incorrect format"))?
                    .iter()
                    .map(|r| {
                        Ok(r.as_str()
                            .ok_or_else(|| anyhow!("Incorrect format"))?
                            .to_string())
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .collect::<Result<Vec<_>>>()?;
        let airport_codes: &[AirportCode] = raw[1][2..raw[1].len() - 5].as_ref();
        let flights = raw[4..]
            .iter()
            .map(|r| {
                Ok({
                    let mut airports: HashMap<String, Vec<AirportCode>> = HashMap::new();
                    r[2..r.len() - 5]
                        .iter()
                        .cloned()
                        .map(|cl| {
                            if !cl.is_empty() {
                                cl.split(", ").map(|a| a.to_string()).collect::<Vec<_>>()
                            } else {
                                vec![]
                            }
                        })
                        .zip(airport_codes.to_owned())
                        .for_each(|(fs, a)| {
                            for f in fs {
                                airports
                                    .entry(f)
                                    .and_modify(|v| v.push(a.to_owned()))
                                    .or_insert_with(|| vec![a.to_owned()]);
                            }
                        });
                    airports
                        .into_iter()
                        .map(|(a, f)| FlightDataFlight {
                            airline: r[1].clone(),
                            flight_number: a,
                            airports: f,
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Ok(FlightData {
            flights,
            airport_codes: airport_codes
                .iter()
                .cloned()
                .filter(|s| !s.is_empty())
                .collect(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        })
    }
    pub fn preprocess(&mut self, config: &mut Config) -> Result<()> {
        info!("Preprocessing flight data");
        debug!("Throwing out ignored airlines");
        self.flights
            .retain(|f| !config.ignored_airlines().contains(&f.airline));

        debug!("Checking airport codes");
        config
            .gates()?
            .iter()
            .map(|g| g.airport.to_owned())
            .sorted()
            .dedup()
            .filter(|a| !self.airport_codes.contains(a))
            .for_each(|a| {
                warn!("Airport `{}` doesn't exist", a);
            });
        let mut airline_airports = config.gates()?.into_iter().map(|g| g.airport);
        config
            .hubs()?
            .into_iter()
            .filter(|a| !airline_airports.contains(a))
            .for_each(|a| {
                warn!("Airport `{}` has no gates but is stated as a hub", a);
            });

        debug!("Ensuring flight number allocations for hubs");
        let fnr_not_specified = config
            .hubs()?
            .into_iter()
            .filter(|a| !config.range_h2n.keys().contains(a))
            .collect::<Vec<_>>();
        if !fnr_not_specified.is_empty() {
            return Err(anyhow!(
                "Flight number range not specified for: {}",
                fnr_not_specified.join(", ")
            ));
        }
        Ok(())
    }
    pub fn num_flights(&self, airport1: &AirportCode, airport2: &AirportCode) -> usize {
        self.flights
            .iter()
            .filter(|f| f.airports.contains(airport1) && f.airports.contains(airport2))
            .count()
    }
}
