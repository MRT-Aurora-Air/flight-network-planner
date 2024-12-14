use std::{
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};
use itertools::Itertools;
use log::{debug, info, warn};

use crate::types::{config::Config, *};

const GATELOGUE_URL: &str = "https://raw.githubusercontent.com/MRT-Map/gatelogue/refs/heads/dist/data_no_sources.json";

#[allow(dead_code)]
#[derive(Debug)]
pub struct FlightDataFlight {
    pub airline: AirlineName,
    pub flight_number: SmolStr,
    pub airports: Vec<AirportCode>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct FlightData {
    pub flights: Vec<FlightDataFlight>,
    pub old_world_airports: Vec<AirportCode>,
    pub new_world_airports: Vec<AirportCode>,
    pub timestamp: u64,
}
impl FlightData {
    pub fn from_gatelogue() -> Result<Self> {
        info!("Downloading gatelogue data");
        let bytes = reqwest::blocking::get(GATELOGUE_URL)?.bytes()?;
        let data = serde_json::from_slice::<serde_json::Value>(&bytes)?;
        let data = data.get("nodes").ok_or(anyhow!("No `nodes`"))?
            .as_object().ok_or(anyhow!("`nodes` not object"))?;

        macro_rules! filter_from_data {
            ($data:ident, $ty:literal) => {
                $data.values()
                    .map(|a| Ok((
                        a,
                        a.get("type").ok_or(anyhow!("No `type`"))?
                            .as_str().ok_or(anyhow!("`type` not string"))?
                    ))
                    )
                    .filter_ok(|(_, ty)| *ty == $ty)
                    .map_ok(|(a, _)| a)
                    .collect::<Result<Vec<_>>>()?.into_iter()
            }
        }

        info!("Processing gatelogue data");
        let flights = filter_from_data!(data, "AirFlight")
            .map(|a| {
                let airline_id = a.get("airline").ok_or(anyhow!("No `airline`"))?
                    .as_number().ok_or(anyhow!("`airline` not number"))?;
                let airline_name = data.get(
                    &airline_id.to_string()
                ).ok_or(anyhow!("airline {airline_id} does not exist"))?
                    .get("name").ok_or(anyhow!("No `name`"))?
                    .as_str().ok_or(anyhow!("`name` not string"))?;

                let flight_number = a.get("codes").ok_or(anyhow!("No `codes`"))?
                    .get(0).ok_or(anyhow!("No codes"))?
                    .as_str().ok_or(anyhow!("`codes` elements not string"))?;

                let airport_codes = a.get("gates").ok_or(anyhow!("No `gates`"))?
                    .as_array().ok_or(anyhow!("`gates` not array"))?
                    .iter().map(|a| {
                    let gate_id = a.as_number().ok_or(anyhow!("`gates` elements not number"))?;
                    let airport_id = data.get(&gate_id.to_string()).ok_or(anyhow!("gate {gate_id} does not exist"))?
                        .get("airport").ok_or(anyhow!("No `airport`"))?
                        .as_number().ok_or(anyhow!("`airport` not number"))?;
                    let airport_code = data.get(&airport_id.to_string()).ok_or(anyhow!("airport {airport_id} does not exist"))?
                        .get("code").ok_or(anyhow!("No `code`"))?
                        .as_str().ok_or(anyhow!("`code` not string"))?;
                    Ok(airport_code.into())
                }).collect::<Result<Vec<_>>>()?;


                Ok(FlightDataFlight {
                    airline: airline_name.into(),
                    flight_number: flight_number.into(),
                    airports: airport_codes,
                })
            }).collect::<Result<Vec<_>>>()?;

        let old_world_airports = filter_from_data!(data, "AirAirport")
            .map(|a| Ok((a, a.get("world").ok_or(anyhow!("No `world`"))?
                .as_str().unwrap_or("New"))))
            .filter_ok(|(_, world)| *world == "Old")
            .map_ok(|(a, _)| Ok(a.get("code").ok_or(anyhow!("No `code`"))?
                .as_str().ok_or(anyhow!("`code` not string"))?.into())).collect::<Result<Result<Vec<_>>>>()??;

        let new_world_airports = filter_from_data!(data, "AirAirport")
            .map(|a| Ok((a, a.get("world").ok_or(anyhow!("No `world`"))?
                .as_str().unwrap_or("New"))))
            .filter_ok(|(_, world)| *world == "New")
            .map_ok(|(a, _)| Ok(a.get("code").ok_or(anyhow!("No `code`"))?
                .as_str().ok_or(anyhow!("`code` not string"))?.into())).collect::<Result<Result<Vec<_>>>>()??;

        Ok(Self {
            flights,
            old_world_airports,
            new_world_airports,
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
            .filter(|a| {
                !self.new_world_airports.contains(a) && !self.old_world_airports.contains(a)
            })
            .for_each(|a| {
                warn!("Airport `{}` doesn't exist", a);
            });

        let airports = config.airports()?;
        config
            .hubs()?
            .into_iter()
            .filter(|a| !airports.iter().contains(a))
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
