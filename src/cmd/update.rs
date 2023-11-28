use std::{collections::HashMap, path::PathBuf};

use anyhow::{anyhow, Result};
use regex::Regex;

use crate::{
    types::{flight::Flight, flight_type::FlightType, fng::FlightNumberGenerator},
    Config,
};

pub fn update(
    old_file: PathBuf,
    generated_plan: Vec<Flight>,
    config: &mut Config,
) -> Result<Vec<Flight>> {
    let old_plan = load_from_out(old_file)?;
    let mut new_plan = vec![];
    let mut used_flight_numbers = vec![];
    let mut flight_number_mapping = HashMap::new();
    let mut new_flights = vec![];

    let mut h2h_fng = FlightNumberGenerator::new(config.range_h2h.to_owned());
    let mut h2n_fng = HashMap::new();
    let mut n2n_fng = FlightNumberGenerator::new(config.range_n2n.to_owned());

    for flight in generated_plan {
        if let Some(old_flight) = old_plan
            .iter()
            .find(|f| f.airport1 == flight.airport1 && f.airport2 == flight.airport2)
        {
            used_flight_numbers.push(old_flight.flight_number.to_owned());
            new_plan.push(Flight {
                flight_number: old_flight.flight_number,
                airport1: flight.airport1,
                airport2: flight.airport2,
                size: flight.size,
                score: flight.score,
                flight_type: flight.flight_type,
            });
        } else {
            new_flights.push(flight);
        }
    }

    for flight in new_flights {
        let flight_number = flight_number_mapping
            .entry(flight.flight_number)
            .or_insert_with(|| {
                let fng = match flight.flight_type {
                    FlightType::ExistingH2H | FlightType::NonExistingH2H => &mut h2h_fng,
                    FlightType::ExistingH2N | FlightType::NonExistingH2N => h2n_fng
                        .entry(
                            (if config
                                .range_h2n
                                .contains_key(&*flight.airport1.0.to_owned())
                            {
                                &flight.airport1.1
                            } else {
                                &flight.airport2.1
                            })
                            .to_owned(),
                        )
                        .or_insert_with(|| {
                            FlightNumberGenerator::new(
                                config
                                    .range_h2n
                                    .get(&*flight.airport1.0.to_owned())
                                    .unwrap_or_else(|| {
                                        &config
                                            .range_h2n[&*flight.airport2.0.to_owned()]
                                    })
                                    .to_owned(),
                            )
                        }),
                    FlightType::ExistingN2N | FlightType::NonExistingN2N => &mut n2n_fng,
                };

                let mut fn_ = fng.next();
                while used_flight_numbers.contains(&fn_.unwrap()) {
                    fn_ = fng.next();
                }
                fn_.unwrap()
            })
            .to_owned();
        used_flight_numbers.push(flight_number.to_owned());
        new_plan.push(Flight {
            flight_number,
            airport1: flight.airport1,
            airport2: flight.airport2,
            size: flight.size,
            score: flight.score,
            flight_type: flight.flight_type,
        });
    }
    Ok(new_plan)
}

pub fn load_from_out(out: PathBuf) -> Result<Vec<Flight>> {
    let regex = Regex::new(r"(\d+) \((.*)\): (...) (.+) (...) (.+) \((\d+), (.2..)\)")?;
    std::fs::read_to_string(out)?
        .split('\n')
        .filter(|l| !l.is_empty())
        .map(|l| {
            Some({
                let re = regex.captures(l)?;

                Flight {
                    flight_number: re.get(1)?.as_str().parse::<u16>().unwrap(),
                    airport1: (re.get(3)?.as_str().into(), re.get(4)?.as_str().into()),
                    airport2: (re.get(5)?.as_str().into(), re.get(6)?.as_str().into()),
                    size: re.get(2)?.as_str().into(),
                    score: re.get(7)?.as_str().parse::<i8>().unwrap(),
                    flight_type: match re.get(8)?.as_str() {
                        "H2Hn" => FlightType::NonExistingH2H,
                        "H2Nn" => FlightType::NonExistingH2N,
                        "N2Nn" => FlightType::NonExistingN2N,
                        "H2He" => FlightType::ExistingH2H,
                        "H2Ne" => FlightType::ExistingH2N,
                        "N2Ne" => FlightType::ExistingN2N,
                        _ => unreachable!(),
                    },
                }
            })
        })
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| anyhow!("Invalid out file"))
}
