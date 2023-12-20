use std::{
    collections::HashMap,
    io::Read,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};
use itertools::Itertools;
use log::{debug, info, warn};

use crate::types::{config::Config, *};

const PLANE_DATA_URL: &str = "https://docs.google.com/spreadsheets/d/1wzvmXHQZ7ee7roIvIrJhkP6oCegnB8-nefWpd8ckqps/export?format=csv&gid=248317803";

// https://stackoverflow.com/questions/64498617/how-to-transpose-a-vector-of-vectors-in-rust
fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>> {
    let len = v[0].len();
    let mut iters: Vec<_> = v
        .into_iter()
        .map(std::iter::IntoIterator::into_iter)
        .collect();
    (0..len)
        .map(|_| {
            iters
                .iter_mut()
                .map(|n| n.next().unwrap())
                .collect::<Vec<T>>()
        })
        .collect()
}

#[derive(Debug)]
pub struct FlightDataFlight {
    pub airline: AirlineName,
    pub flight_number: SmolStr,
    pub airports: Vec<AirportCode>,
}

#[derive(Debug)]
pub struct FlightData {
    pub flights: Vec<FlightDataFlight>,
    pub old_world_airports: Vec<AirportCode>,
    pub new_world_airports: Vec<AirportCode>,
    pub timestamp: u64,
}
impl FlightData {
    pub fn from_sheets() -> Result<Self> {
        let mut str = String::new();
        reqwest::blocking::get(PLANE_DATA_URL)?.read_to_string(&mut str)?;
        let mut csv = csv::ReaderBuilder::default()
            .has_headers(false)
            .from_reader(str.as_bytes());
        let raw = transpose(
            csv.records()
                .map(|record| Ok(record?.into_iter().map(SmolStr::from).collect::<Vec<_>>()))
                .collect::<Result<Vec<_>>>()?,
        );
        let airport_codes: &[AirportCode] = &raw[1][2..raw[1].len() - 5];
        let locations: &[SmolStr] = &raw[2][2..raw[2].len()];
        let flights = raw[4..]
            .iter()
            .map(|r| {
                Ok({
                    let mut airports: HashMap<SmolStr, Vec<AirportCode>> = HashMap::new();
                    r[2..r.len() - 5]
                        .iter()
                        .cloned()
                        .map(|cl| {
                            if cl.is_empty() {
                                vec![]
                            } else {
                                cl.split(", ")
                                    .map(std::string::ToString::to_string)
                                    .collect::<Vec<_>>()
                            }
                        })
                        .zip(airport_codes.to_owned())
                        .for_each(|(fs, a)| {
                            for f in fs {
                                airports
                                    .entry(f.into())
                                    .and_modify(|v| v.push(a.to_owned()))
                                    .or_insert_with(|| vec![a.to_owned()]);
                            }
                        });
                    airports
                        .into_iter()
                        .map(|(a, f)| FlightDataFlight {
                            airline: r[1].to_owned(),
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

        let filter_codes = |world: &'static str| {
            airport_codes
                .iter()
                .cloned()
                .zip(locations.iter().cloned())
                .filter(|(s, l)| !s.is_empty() && !l.is_empty())
                .filter_map(|(s, l)| (l.trim() == world).then_some(s))
                .collect::<Vec<_>>()
        };

        Ok(Self {
            flights,
            old_world_airports: filter_codes("Old"),
            new_world_airports: filter_codes("New"),
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
