use crate::config::Config;
use crate::flight::Flight;
use crate::flight_type::FlightType;
use crate::fng::FlightNumberGenerator;
use crate::gate::Gate;
use crate::types::*;
use crate::FlightData;
use anyhow::{anyhow, Result};
use itertools::Itertools;
use log::{debug, info, trace};
use std::collections::HashMap;

fn sort_gates(x: Vec<(Gate, Gate, i8, FlightType)>, config: &mut Config, fd: &FlightData, old_plan: &Option<Vec<Flight>>) -> Result<Vec<(Gate, Gate, i8, FlightType)>> {
    Ok(x.into_iter()
        .map(|(g1, g2, _, ty)| {
            let mut s = (&g1, &g2).score(config, fd)?;
            if let Some(old_plan) = old_plan {
                if old_plan.iter().filter(|f| f.airport1 == (g1.airport.to_owned(), g1.code.to_owned())
                    && f.airport2 == (g2.airport.to_owned(), g2.code.to_owned())).count() > 0 {
                    s += 1;
                }
            }
            Ok((g1, g2, s, ty))
        })
        .collect::<Result<Vec<_>>>()?.into_iter()
        .sorted_by(|(_, _, s1, _), (_, _, s2, _)| s1.cmp(s2))
        .collect::<Vec<_>>())
}

pub fn run(config: &mut Config, fd: &FlightData, old_plan: &Option<Vec<Flight>>) -> Result<Vec<Flight>> {
    let hubs = config.hubs()?;
    let restricted_between = config.restricted_between.to_owned();
    let restricted_to = config.restricted_to.to_owned();
    let gate_allowed_dests = config.gate_allowed_dests.to_owned();
    let gate_denied_dests = config.gate_denied_dests.to_owned();
    let no_dupes = config.no_dupes.to_owned();
    let mut possible_flights = config
        .gates()?
        .into_iter()
        .tuple_combinations::<(_, _)>()
        .filter(|(g1, g2)| {
            !restricted_between.iter().any(|re| {
                vec![g1.airport.to_owned(), g2.airport.to_owned()]
                    .into_iter()
                    .all(|a| re.contains(&a))
            })
        })
        .filter(|(g1, g2)| g1.airport != g2.airport && g1.size == g2.size)
        .filter(|(g1, g2)| {
            !restricted_to
                .get(&*g1.airport)
                .unwrap_or(&vec![])
                .contains(&g2.airport)
        })
        .filter(|(g1, g2)| {
            !restricted_to
                .get(&*g2.airport)
                .unwrap_or(&vec![])
                .contains(&g1.airport)
        })
        .filter(|(g1, g2)| {
            if let Some(gates) = gate_allowed_dests.get(&*g1.airport) {
                if let Some(gate) = gates.get(&*g1.code) {
                    gate.contains(&g2.airport)
                } else {
                    true
                }
            } else {
                true
            }
        })
        .filter(|(g1, g2)| {
            if let Some(gates) = gate_allowed_dests.get(&*g2.airport) {
                if let Some(gate) = gates.get(&*g2.code) {
                    gate.contains(&g1.airport)
                } else {
                    true
                }
            } else {
                true
            }
        })
        .filter(|(g1, g2)| {
            if let Some(gates) = gate_denied_dests.get(&*g1.airport) {
                if let Some(gate) = gates.get(&*g1.code) {
                    !gate.contains(&g2.airport)
                } else {
                    true
                }
            } else {
                true
            }
        })
        .filter(|(g1, g2)| {
            if let Some(gates) = gate_denied_dests.get(&*g2.airport) {
                if let Some(gate) = gates.get(&*g2.code) {
                    !gate.contains(&g1.airport)
                } else {
                    true
                }
            } else {
                true
            }
        })
        .map(|(g1, g2)| {
            let ty = (&g1, &g2).get_flight_type(config, fd)?;
            Ok((g1, g2, 0i8, ty))
        })
        .filter_ok(|(g1, g2, _, ty)| {
            if no_dupes.contains(&g1.airport) || no_dupes.contains(&g2.airport) {
                ![
                    FlightType::ExistingH2H,
                    FlightType::ExistingH2N,
                    FlightType::ExistingN2N,
                ]
                    .contains(ty)
            } else { true }
        })
        .collect::<Result<Vec<_>>>()?;

    let mut h2h_fng = FlightNumberGenerator::new(config.range_h2h.to_owned());
    let mut h2n_fng = HashMap::new();
    let mut n2n_fng = FlightNumberGenerator::new(config.range_n2n.to_owned());

    let mut destinations: HashMap<Gate, Vec<AirportCode>> = HashMap::new();
    let mut flights: Vec<Flight> = vec![];

    possible_flights = sort_gates(possible_flights, config, fd, old_plan)?;

    while let Some((g1, g2, mut s, ty)) = possible_flights.pop() {
        if destinations.get(&g1).unwrap_or(&vec![]).len() >= *config.max_dests_per_gate.get(&g1.airport).unwrap_or(&u8::MAX) as usize
        || destinations.get(&g2).unwrap_or(&vec![]).len() >= *config.max_dests_per_gate.get(&g2.airport).unwrap_or(&u8::MAX) as usize {
            continue;
        }
        s -= (destinations.get(&g1).unwrap_or(&vec![]).len() as i8).min(destinations.get(&g2).unwrap_or(&vec![]).len() as i8);
        if s < 0 {
            continue;
        }
        let max1 = match ty {
            FlightType::ExistingH2H | FlightType::NonExistingH2H => config.max_h2h,
            FlightType::ExistingH2N | FlightType::NonExistingH2N => {
                if hubs.contains(&g1.airport) {
                    config.max_h2n_hub
                } else {
                    config.max_h2n_nonhub
                }
            }
            FlightType::ExistingN2N | FlightType::NonExistingN2N => config.max_n2n,
        };
        let max2 = match ty {
            FlightType::ExistingH2H | FlightType::NonExistingH2H => config.max_h2h,
            FlightType::ExistingH2N | FlightType::NonExistingH2N => {
                if hubs.contains(&g2.airport) {
                    config.max_h2n_hub
                } else {
                    config.max_h2n_nonhub
                }
            }
            FlightType::ExistingN2N | FlightType::NonExistingN2N => config.max_n2n,
        };

        if flights.iter().any(|f| {
            (f.airport1.0 == g1.airport && f.airport2.0 == g2.airport)
                || (f.airport1.0 == g2.airport && f.airport2.0 == g1.airport)
        }) {
            trace!(
                "Rejected ({} {}): {} {} <-> {} {} (already exists)",
                ty,
                g1.size,
                g1.airport,
                g1.code,
                g2.airport,
                g2.code
            );
            continue;
        }

        let g1_hardmax = (if hubs.contains(&g1.airport) {
            config.hard_max_hub
        } else {
            config.hard_max_nonhub
        }) as usize;
        if destinations.get(&g1).unwrap_or(&vec![]).len() >= g1_hardmax {
            debug!(
                "Rejected ({} {}): {} {} <-> {} {} ({2} hit max limit of {})",
                ty, g2.size, g1.airport, g1.code, g2.airport, g2.code, g1_hardmax
            );
            continue;
        }
        let g2_hardmax = (if hubs.contains(&g2.airport) {
            config.hard_max_hub
        } else {
            config.hard_max_nonhub
        }) as usize;
        if destinations.get(&g2).unwrap_or(&vec![]).len() >= g2_hardmax {
            debug!(
                "Rejected ({} {}): {} {} <-> {} {} ({2} hit max limit of {})",
                ty, g1.size, g2.airport, g2.code, g1.airport, g1.code, g2_hardmax
            );
            continue;
        }
        if destinations
            .get(&g1)
            .unwrap_or(&vec![])
            .iter()
            .filter(|d| (&g1.airport, *d).get_flight_type(config, fd).unwrap() == ty)
            .count()
            >= max1 as usize
        {
            debug!(
                "Rejected ({} {}): {} {} <-> {} {} ({2} hit max type limit of {})",
                ty, g2.size, g1.airport, g1.code, g2.airport, g2.code, max1
            );
            continue;
        }
        if destinations
            .get(&g2)
            .unwrap_or(&vec![])
            .iter()
            .filter(|d| (&g2.airport, *d).get_flight_type(config, fd).unwrap() == ty)
            .count()
            >= max2 as usize
        {
            debug!(
                "Rejected ({} {}): {} {} <-> {} {} ({2} hit max type limit of {})",
                ty, g1.size, g2.airport, g2.code, g1.airport, g1.code, max2
            );
            continue;
        }

        destinations
            .entry(g1.to_owned())
            .or_insert(vec![])
            .push(g2.airport.to_owned());
        destinations
            .entry(g2.to_owned())
            .or_insert(vec![])
            .push(g1.airport.to_owned());

        let fng = match ty {
            FlightType::ExistingH2H | FlightType::NonExistingH2H => &mut h2h_fng,
            FlightType::ExistingH2N | FlightType::NonExistingH2N => h2n_fng
                .entry(
                    (if config.range_h2n.contains_key(&*g1.airport.to_owned()) {
                        &g1
                    } else {
                        &g2
                    })
                    .airport
                    .to_owned(),
                )
                .or_insert_with(|| {
                    FlightNumberGenerator::new(
                        config
                            .range_h2n
                            .get(&*g1.airport.to_owned())
                            .unwrap_or_else(|| {
                                config.range_h2n.get(&*g2.airport.to_owned()).unwrap()
                            })
                            .to_owned(),
                    )
                }),
            FlightType::ExistingN2N | FlightType::NonExistingN2N => &mut n2n_fng,
        };

        let fn1 = fng.next();
        let fn2 = if config.both_dir_same_num {
            fn1
        } else {
            fng.next()
        };

        let flight1 = Flight {
            flight_number: if let Some(fn_) = fn1 {
                fn_
            } else {
                return Err(anyhow!(
                    "Could not generate flight number for {} -> {}",
                    g1.airport,
                    g2.airport
                ));
            },
            airport1: (g1.airport.to_owned(), g1.code.to_owned()),
            airport2: (g2.airport.to_owned(), g2.code.to_owned()),
            size: g1.size.to_owned(),
            score: s,
            flight_type: ty
        };
        info!(
            "{} ({} {}): {} {} -> {} {}, {}",
            flight1.flight_number, ty, g1.size, g1.airport, g1.code, g2.airport, g2.code, s
        );
        flights.push(flight1);

        let flight2 = Flight {
            flight_number: if let Some(fn_) = fn2 {
                fn_
            } else {
                return Err(anyhow!(
                    "Could not generate flight number for {} -> {}",
                    g2.airport,
                    g1.airport
                ));
            },
            airport1: (g2.airport.to_owned(), g2.code.to_owned()),
            airport2: (g1.airport.to_owned(), g1.code.to_owned()),
            size: g2.size.to_owned(),
            score: s,
            flight_type: ty,
        };
        info!(
            "{} ({} {}): {} {} -> {} {}, {}",
            flight2.flight_number, ty, g2.size, g2.airport, g2.code, g1.airport, g1.code, s
        );
        flights.push(flight2);
        //possible_flights = sort_gates(possible_flights, config, fd)?;
    }

    Ok(flights)
}
