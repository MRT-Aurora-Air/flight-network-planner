use crate::config::Config;
use crate::types::*;
use crate::FlightData;
use anyhow::Result;

pub fn run(config: &mut Config) -> Result<Vec<Flight>> {
    let mut fd = FlightData::from_sheets()?;
    fd.preprocess(config)?;
    Ok(vec![])
}
