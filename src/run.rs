use std::collections::HashMap;
use crate::config::Config;
use crate::types::*;
use crate::FlightData;
use anyhow::Result;
use itertools::Itertools;
use log::{info, warn};

pub fn run(mut config: Config) -> Result<Vec<Flight>> {
    let mut fd = FlightData::from_sheets()?;
    fd.preprocess(&mut config)?;
    let mut gates = config.gates()?;
    let hubs = config.hubs()?;
    let restricted_between = config.restricted_between.to_owned();
    let restricted_to = config.restricted_to.to_owned();

    let mut h2h_fng = FlightNumberGenerator::new(config.range_h2h.to_owned());
    let mut h2n_fng = HashMap::new();
    let mut n2n_fng = FlightNumberGenerator::new(config.range_n2n.to_owned());

    let mut destinations = HashMap::new();

    Ok(gates
        .iter()
        .tuple_combinations::<(_, _)>()
        .filter(|(g1, g2)| {
            !restricted_between
                .iter()
                .any(|re| vec![g1.airport.to_owned(), g2.airport.to_owned()].into_iter().all(|a| re.contains(&a)))
        })
        .filter(|(g1, g2)| g1.airport != g2.airport && g1.size == g2.size)
        .filter(|(g1, g2)| !restricted_to.get(&*g1.airport).unwrap_or(&vec![]).contains(&g2.airport))
        .filter(|(g1, g2)| !restricted_to.get(&*g2.airport).unwrap_or(&vec![]).contains(&g1.airport))
        .map(|(g1, g2)| {
            let s = (g1, g2).score(&mut config, &fd)?;
            let ty = (g1, g2).get_flight_type(&mut config, &fd)?;
            Ok(
                (g1, g2, s, ty)
            )
        })
        .collect::<Result<Vec<_>>>()?.into_iter()
        .filter(|(_, _, score, _)| *score >= 0)
        .sorted_by(|(_, _, s1, ty1), (_, _, s2, ty2)| {
            if ty2 == ty1 {
                s2.cmp(s1)
            } else {
                ty1.cmp(ty2)
            }
        }).filter_map(|(g1, g2, _, ty)| {
        let max = match ty {
            FlightType::ExistingH2H | FlightType::NonExistingH2H => config.max_h2h,
            FlightType::ExistingH2N | FlightType::NonExistingH2N => config.max_h2n,
            FlightType::ExistingN2N | FlightType::NonExistingN2N => config.max_n2n,
        };
        if destinations.get(&*g1.airport.to_owned()).unwrap_or(&vec![]).len()
            >= (if hubs.contains(&g1.airport) { config.hard_max_hub } else { config.hard_max_nonhub }) as usize
        || destinations.get(&*g2.airport.to_owned()).unwrap_or(&vec![]).len()
            >= (if hubs.contains(&g2.airport) { config.hard_max_hub } else { config.hard_max_nonhub }) as usize
        || destinations.get(&*g1.airport.to_owned()).unwrap_or(&vec![]).iter()
            .filter(|d| (&g1.airport, *d).get_flight_type(&mut config, &fd).unwrap() == ty).count() >= max as usize
        || destinations.get(&*g2.airport.to_owned()).unwrap_or(&vec![]).iter()
            .filter(|d| (&g2.airport, *d).get_flight_type(&mut config, &fd).unwrap() == ty).count() >= max as usize {
            return None;
        }
        destinations.entry(g1.airport.to_owned()).or_insert(vec![]).push(g2.airport.to_owned());
        destinations.entry(g2.airport.to_owned()).or_insert(vec![]).push(g1.airport.to_owned());
        let flight = Flight {
            flight_number: if let Some(fn_) = match ty {
                FlightType::ExistingH2H | FlightType::NonExistingH2H => &mut h2h_fng,
                FlightType::ExistingH2N | FlightType::NonExistingH2N => h2n_fng.entry(g1.airport.to_owned())
                    .or_insert_with(|| FlightNumberGenerator::new(config.range_h2n.get(&*g1.airport.to_owned()).unwrap_or_else(|| config.range_h2n.get(&*g2.airport.to_owned()).unwrap()).to_owned())),
                FlightType::ExistingN2N | FlightType::NonExistingN2N => &mut n2n_fng,
            }.next() {fn_} else {
                warn!("Could not generate flight number for {} -> {}", g1.airport, g2.airport);
                0
            },
            airport1: (g1.airport.to_owned(), g1.code.to_owned()),
            airport2: (g2.airport.to_owned(), g2.code.to_owned())
        }; // TODO flight for other dir
        info!("{} ({}): {} -> {}", flight.flight_number, g1.size, g1.airport, g2.airport);
        Some(flight)
    }).collect::<Vec<_>>())
}
