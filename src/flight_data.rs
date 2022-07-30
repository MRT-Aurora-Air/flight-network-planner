use crate::types::*;
use anyhow::anyhow;
use lazy_static::lazy_static;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

const PLANE_DATA_URL: &str = "https://sheets.googleapis.com/v4/spreadsheets/1wzvmXHQZ7ee7roIvIrJhkP6oCegnB8-nefWpd8ckqps/values/Airline+Class+Distribution?majorDimension=COLUMNS&key=AIzaSyCCRZqIOAVfwBNUofWbrkz0q5z4FUaCUyE";

#[derive(Debug)]
pub struct Flight {
    pub airline: AirlineName,
    pub flight_number: FlightNumber,
    pub airports: Vec<(AirportCode, Option<GateCode>)>,
}
#[derive(Debug)]
pub struct FlightData {
    flights: Vec<Flight>,
    airport_codes: Vec<AirportCode>,
    timestamp: u64,
}
impl FlightData {
    pub fn from_sheets() -> anyhow::Result<Self> {
        let raw = reqwest::blocking::get(PLANE_DATA_URL)?.json::<Value>()?["values"]
            .as_array()
            .ok_or(anyhow!("Incorrect format"))?
            .into_iter()
            .map(|r| {
                r.as_array()
                    .ok_or(anyhow!("Incorrect format"))?
                    .into_iter()
                    .map(|r| Ok(r.as_str().ok_or(anyhow!("Incorrect format"))?.to_string()))
                    .collect::<anyhow::Result<Vec<_>>>()
                    .into()
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        let airport_codes: &[AirportCode] = raw[1][2..raw[1].len() - 5].as_ref();
        let flights = raw[4..]
            .iter()
            .map(|r| {
                Ok({
                    let mut airports: HashMap<FlightNumber, Vec<AirportCode>> = HashMap::new();
                    r[2..r.len() - 5]
                        .iter()
                        .cloned()
                        .map(|cl| {
                            cl.split(", ") // TODO acc for empty strings
                                .map(|f| Ok(f.parse::<FlightNumber>()?))
                                .collect::<anyhow::Result<Vec<_>>>()
                        })
                        .zip(airport_codes.to_owned())
                        .map(|(f, a)| {
                            for f in f? {
                                airports.entry(f)
                                    .and_modify(|v| v.push(a.to_owned()))
                                    .or_insert(vec![a.to_owned()]);
                            }
                            Ok(())
                        })
                        .collect::<anyhow::Result<_>>()?;
                    airports.into_iter()
                        .map(|(a, f)| Flight {
                            airline: r[1].clone(),
                            flight_number: a,
                            airports: f.into_iter()
                                .map(|f| (f, None))
                                .collect()
                        }).collect::<Vec<_>>()
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?.into_iter().flatten().collect::<Vec<_>>();

        Ok(FlightData {
            flights,
            airport_codes: airport_codes
                .iter()
                .cloned()
                .filter(|s| !s.is_empty())
                .collect(),
            timestamp: Instant::now().elapsed().as_secs(),
        })
    }
}