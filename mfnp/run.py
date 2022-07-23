from __future__ import annotations

import itertools
from typing import Tuple, Generator

import blessed
import json

term = blessed.Terminal()

import mfnp.utils as utils


def flight_already_exists(
    data1: utils._FlightData,
    data2: list[utils.FlightRoute],
    c1: str,
    c2: str,
    dupe_ok: bool = False,
) -> bool:
    for flight_set in data1.values():
        for flight_flights in flight_set.values():
            if c1 in flight_flights and c2 in flight_flights and not dupe_ok:
                return True
    for flight in data2:
        if flight.orig == c1 and flight.dest == c2:
            return True
    return False


def flight_num_generator(
    config: utils.Config,
    flight_nums: list[utils.FlightNumber],
    mode: str,
    code_: str | None = None,
) -> Generator[utils.FlightNumber, None, None]:
    nums = config.__getattribute__("range_" + mode)
    if mode == "h2n":
        nums = nums[code_]
    nums = [range(r[0], r[1] + 1) for r in nums]
    for num in itertools.chain(*nums):
        if config.airline_code + str(num) not in flight_nums:
            yield config.airline_code + str(num)
    utils._warn(
        f"Not enough flight numbers for {mode} flights"
        + (f" from {code_}" if code_ else "")
    )


def get_connection_type(
    config: utils.Config, origin: str, dests: list[str]
) -> dict[str, int]:
    out = {"h2h": 0, "h2n": 0, "n2n": 0}
    for dest in dests:
        if dest in config.hubs:
            if origin in config.hubs:
                out["h2h"] += 1
            else:
                out["h2n"] += 1
        else:
            if origin in config.hubs:
                out["h2n"] += 1
            else:
                out["h2n"] += 1
    return out


def get_gate(
    config: utils.Config,
    gates: dict[str, list[utils.Gate]],
    nonhubs: list[utils.AirportCode],
    data1: utils._FlightData,
    data2: list[utils.FlightRoute],
    c1: str,
    c2: str,
    exist_ok: bool = False,
) -> Tuple[str | None, str | None, str | None]:
    if flight_already_exists(data1, data2, c1, c2, exist_ok):
        return None, None, None
    g1s = gates[c1][:]
    g2s = gates[c2][:]
    c1_is_hub = c1 not in nonhubs
    c2_is_hub = c2 not in nonhubs
    for gs in [g1s, g2s]:
        for g in gs:
            g.score = 0
            if len(g.dests) == 0:
                g.score = config.hard_max // 2
            for dest in g.dests:
                if not flight_already_exists(data1, data2, g.code, dest, exist_ok):
                    g.score += 1
            c = c1 if gs is g1s else c2
            con_types = get_connection_type(config, c, g.dests)
            if c in nonhubs:
                if con_types["h2n"] >= config.max_h2n and (
                    (c1_is_hub and not c2_is_hub) or (not c1_is_hub and c2_is_hub)
                ):
                    g.score -= 4 * (con_types["h2n"] - config.max_h2n + 1)
                if con_types["n2n"] >= config.max_n2n and (
                    not c1_is_hub and not c2_is_hub
                ):
                    g.score -= 4 * (con_types["n2n"] - config.max_n2n + 1)

    combis = []
    for g1 in g1s:
        for g2 in g2s:
            combis.append((g1, g2))
    combis.sort(key=lambda x: x[0].score + x[1].score, reverse=True)
    for g1, g2 in combis:
        if g1.score + g2.score < 0:
            continue
        if g1.size == g2.size and g1.capacity > 0 and g2.capacity > 0:
            g1.capacity -= 1
            g2.capacity -= 1
            g1.dests.append(g2.code)
            g2.dests.append(g1.code)
            return g1.size, g1.code, g2.code
    return None, None, None


def run(config: utils.Config, output_format: str = "json", nocache: bool = False):
    # == Preparatory stuff ==
    # fill in defaults
    utils._log("Filling in defaults")
    if not config.ignored_airlines:
        config.ignored_airlines.append(config.airline_name)
    if config.gate_json:
        with open(config.gate_json, "r") as f:
            config.gates = json.load(f)
    if not config.hubs:
        for airport, gates in config.gates.items():
            if len(gates) >= config.hub_threshold:
                config.hubs.append(airport)

    # get data from the transit sheet
    data, airports = utils.get_flight_data(nocache=nocache)

    utils._log("Preprocessing data")

    # throw out ignored airlines
    utils._debug("Throwing out ignored airlines")
    for airline in config.ignored_airlines:
        if airline in data:
            del data[airline]
        else:
            utils._warn(
                f"Airline `{airline}` doesn't exist, maybe you mean: {utils._gcm(airline, data.keys())}"
            )

    # double-check all airport codes
    utils._debug("Double-checking airport codes")
    for code in config.gates.keys():
        if code not in airports:
            utils._warn(
                f"Airport `{code}` doesn't exist, maybe you mean: {utils._gcm(code, airports)}"
            )
    for code in config.hubs:
        if code not in config.gates.keys():
            utils._warn(
                f"Airport `{code}` has no gates but is stated as a hub, maybe you mean: {utils._gcm(code, config.gates.keys())}"
            )
    config.hubs = [c for c in config.hubs if c in config.gates]

    # ensure that flight numbers are allocated for hubs
    utils._debug("Ensuring flight number allocations for hubs")
    for hub in config.hubs:
        if hub not in config.range_h2n:
            raise ValueError(f"Flight number range not specified for hub `{hub}`")

    # == Plan the flights ==
    utils._log("Planning flights")
    flight_plan: list[utils.FlightRoute] = []
    flight_nums: list[utils.FlightNumber] = []
    nonhubs = [g for g in config.gates.keys() if g not in config.hubs]
    gates: dict[str, list[utils.Gate]] = config.gates.copy()
    for gate_set in gates.values():
        for gate in gate_set:
            gate.capacity = config.hard_max
            gate.dests = []

    for exist_ok in [False, True]:
        # hub-to-non-hub flights
        for code1 in config.hubs:
            utils._log(
                f"Processing H2N flights ({'existing' if exist_ok else 'non-existing'}) for {code1}"
            )
            flight_nums_h2n = flight_num_generator(
                config, flight_nums, "h2n", code_=code1
            )
            for code2 in nonhubs:
                size, gate1, gate2 = get_gate(
                    config,
                    gates,
                    nonhubs,
                    data,
                    flight_plan,
                    code1,
                    code2,
                    exist_ok=exist_ok,
                )
                if size is None:
                    continue
                try:
                    flight_num1 = next(flight_nums_h2n)
                    flight_num2 = (
                        flight_num1
                        if config.both_dir_same_num
                        else next(flight_nums_h2n)
                    )
                    flight_nums.append(flight_num1)
                    flight_nums.append(flight_num2)
                except StopIteration:
                    break
                for origin, dest, origin_gate, dest_gate, flight_num in [
                    (code1, code2, gate1, gate2, flight_num1),
                    (code2, code1, gate2, gate1, flight_num2),
                ]:
                    flight_plan.append(
                        utils.FlightRoute(
                            code=flight_num,
                            orig=origin,
                            orig_gate=origin_gate,
                            dest=dest,
                            dest_gate=dest_gate,
                            size=size,
                        )
                    )
                    utils._debug(
                        f"{flight_num1} (size {size}): {origin} ({origin_gate}) -> {dest} ({dest_gate})"
                    )

        # hub-to-hub flights
        flight_nums_h2h = flight_num_generator(config, flight_nums, "h2h")
        utils._log(
            f"Processing H2H flights ({'existing' if exist_ok else 'non-existing'})"
        )
        for code1, code2 in itertools.combinations(config.hubs, 2):
            size, gate1, gate2 = get_gate(
                config,
                gates,
                nonhubs,
                data,
                flight_plan,
                code1,
                code2,
                exist_ok=exist_ok,
            )
            if size is None:
                continue
            try:
                flight_num1 = next(flight_nums_h2h)
                flight_num2 = (
                    flight_num1 if config.both_dir_same_num else next(flight_nums_h2h)
                )
                flight_nums.append(flight_num1)
                flight_nums.append(flight_num2)
            except StopIteration:
                break
            for origin, dest, origin_gate, dest_gate, flight_num in [
                (code1, code2, gate1, gate2, flight_num1),
                (code2, code1, gate2, gate1, flight_num2),
            ]:
                flight_plan.append(
                    utils.FlightRoute(
                        code=flight_num,
                        orig=origin,
                        orig_gate=origin_gate,
                        dest=dest,
                        dest_gate=dest_gate,
                        size=size,
                    )
                )
                utils._debug(
                    f"{flight_num1} (size {size}): {origin} ({origin_gate}) -> {dest} ({dest_gate})"
                )

        # nonhub-to-nonhub flights
        flight_nums_n2n = flight_num_generator(config, flight_nums, "n2n")
        utils._log(
            f"Processing N2N flights ({'existing' if exist_ok else 'non-existing'})"
        )
        for code1, code2 in itertools.combinations(nonhubs, 2):
            size, gate1, gate2 = get_gate(
                config,
                gates,
                nonhubs,
                data,
                flight_plan,
                code1,
                code2,
                exist_ok=exist_ok,
            )
            if size is None:
                continue
            try:
                flight_num1 = next(flight_nums_n2n)
                flight_num2 = (
                    flight_num1 if config.both_dir_same_num else next(flight_nums_n2n)
                )
                flight_nums.append(flight_num1)
                flight_nums.append(flight_num2)
            except StopIteration:
                break
            for origin, dest, origin_gate, dest_gate, flight_num in [
                (code1, code2, gate1, gate2, flight_num1),
                (code2, code1, gate2, gate1, flight_num2),
            ]:
                flight_plan.append(
                    utils.FlightRoute(
                        code=flight_num,
                        orig=origin,
                        orig_gate=origin_gate,
                        dest=dest,
                        dest_gate=dest_gate,
                        size=size,
                    )
                )
                utils._debug(
                    f"{flight_num1} (size {size}): {origin} ({origin_gate}) -> {dest} ({dest_gate})"
                )

    utils._log("Network planning complete")

    # == convert to flight list ==
    if output_format == "json":
        processed_flights = []
        out = []
        for flight in flight_plan:
            if flight.code in processed_flights:
                continue
            out.append(
                {
                    "number": flight.code,
                    "size": flight.size,
                    "code1": flight.orig,
                    "gate1": flight.orig_gate,
                    "code2": flight.dest,
                    "gate2": flight.dest_gate,
                }
            )
            processed_flights.append(code)
        out.sort(key=lambda x: int(x["number"][len(config.airline_code) :]))

    emptygates = []
    fullgates = []
    for code, gates in gates.items():
        for gate in gates:
            if gate.capacity == config.hard_max:
                emptygates.append(f"{code} gate {gate.code}")
            elif gate.capacity == 0:
                fullgates.append(f"{code} gate {gate.code}")

    print(term.yellow(f"== Flight network stats =="))
    print(term.yellow(f"Flights: {len(out)}"))
    print(term.yellow(f"Destinations: {len(config.gates)}"))
    print(term.yellow(f"Flight:Destination ratio: {len(out) / len(config.gates)}"))
    print(term.yellow(f"Empty gates: {', '.join(emptygates)}"))
    print(term.yellow(f"Full gates: {', '.join(fullgates)}"))

    return out
