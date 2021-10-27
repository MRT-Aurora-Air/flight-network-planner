import itertools
from types import GeneratorType
from typing import Optional, Tuple

import blessed
import json
term = blessed.Terminal()

try:
    import utils
except ModuleNotFoundError:
    import mfnp.utils as utils

def run(config: dict, output_format: str="json", verbosity: int=0, nowarn: bool=False, nocache: bool=False):
    # == Preparatory stuff ==
    # fill in defaults
    utils._log("Filling in defaults", 0, verbosity)
    if not config['ignored_airlines']: config['ignored_airlines'].append(config['airline_name'])
    if config['gate_json']:
        with open(config['gate_json'], 'r') as f:
            config['gates'] = json.load(f)
    if not config['hubs']:
        for airport, gates in config['gates'].items():
            if len(gates) >= config['hub_threshold']: config['hubs'].append(airport)
    #print(config)

    # get data from the transit sheet
    data, airports = utils.get_flight_data(verbosity=verbosity, nocache=nocache)

    utils._log("Preprocessing data", 0, verbosity)

    # throw out ignored airlines
    utils._log("Throwing out ignored airlines", 1, verbosity)
    for airline in config['ignored_airlines']:
        if airline in data: del data[airline]
        else: utils._warn(f"Airline `{airline}` doesn't exist, maybe you mean: {utils._gcm(airline, data.keys())}", nowarn)

    # double-check all airport codes
    utils._log("Double-checking airport codes", 1, verbosity)
    for code in config['gates'].keys():
        if code not in airports:
            utils._warn(f"Airport `{code}` doesn't exist, maybe you mean: {utils._gcm(code, airports)}", nowarn)
    for code in config['hubs']:
        if code not in config['gates'].keys():
            utils._warn(f"Airport `{code}` has no gates but is stated as a hub, maybe you mean: {utils._gcm(code, config['gates'].keys())}", nowarn)
    config['hubs'] = list(filter(lambda c: c in config['gates'].keys(), config['hubs']))

    # ensure that flight numbers are allocated for hubs
    utils._log("Ensuring flight number allocations for hubs", 1, verbosity)
    for hub in config['hubs']:
      if hub not in config['range_h2n']:
        raise ValueError(f"Flight number range not specified for hub `{hub}`")

    # == Plan the flights ==
    utils._log("Planning flights", 0, verbosity)
    flight_plan = {} # {airport_code: {flight_num: {gate: XXX, dest: XXX, dest_gate: XXX, size: X}}}
    flight_nums = []
    nonhubs = [g for g in config['gates'].keys() if g not in config['hubs']]
    gates = config['gates'].copy() # {airport_code: [{code: XXX, size: XXX, capacity: XXX, dests: [XXX, ...]}, ...]}
    for gate_set in gates.values():
        for gate in gate_set:
            gate['capacity'] = 6
            gate['dests'] = []

    def flight_already_exists(c1: str, c2: str) -> bool:
        for flight_set in data.values():
            for flight_flights in flight_set.values():
                if c1 in flight_flights and c2 in flight_flights:
                    return True
        return False

    def flight_num_generator(mode: str, code_: Optional[str]=None) -> str:
        nums = config['range_'+mode]
        if mode == "h2n": nums = nums[code_]
        nums = list(map(lambda r: range(r[0], r[1]+1), nums))
        for num in itertools.chain(*nums):
            if config['airline_code']+str(num) not in flight_nums: 
               yield config['airline_code']+str(num)
        utils._warn(f"Not enough flight numbers for {mode} flights" + (f" from {code_}" if code_ else ""), nowarn)

    def get_gate(c1: str, c2: str, exist_ok: bool=False) -> Tuple[Optional[str], Optional[str], Optional[str]]:
        if not exist_ok and flight_already_exists(c1, c2):
            return None, None, None
        g1s = gates[c1][:]
        g2s = gates[c2][:]
        for gs in [g1s, g2s]:
            for g in gs:
                g['score'] = 0
                for dest in g['dests']:
                    if not flight_already_exists(g, dest):
                        g['score'] += 1
        combis = []
        for g1 in g1s:
            for g2 in g2s:
                combis.append((g1, g2))
        combis.sort(key=lambda x: x[0]['score']+x[1]['score'], reverse=True)
        for g1, g2 in combis:
            if g1['size'] == g2['size'] and g1['capacity'] > 0 and g2['capacity'] > 0:
                g1['capacity'] -= 1
                g2['capacity'] -= 1
                g1['dests'].append(g2)
                g2['dests'].append(g1)
                return g1['size'], g1['code'], g2['code']
        return None, None, None
        

    for exist_ok in [True, False]:
        # hub-to-nonhub flights
        for code1 in config['hubs']:
            utils._log(f"Processing H2N flights ({'existing' if exist_ok else 'non-existing'}) for {code1}", 0, verbosity)
            flight_nums_h2n = flight_num_generator('h2n', code_=code1)
            for code2 in nonhubs:
                if not flight_already_exists(code1, code2):
                    size, gate1, gate2 = get_gate(code1, code2, exist_ok=exist_ok)
                    if size is None: continue
                    try:
                        flight_num1 = next(flight_nums_h2n)
                        flight_num2 = flight_num1 if config['both_dir_same_num'] else next(flight_nums_h2n)
                    except StopIteration:
                        break
                    for origin, dest, origin_gate, dest_gate, flight_num in [(code1, code2, gate1, gate2, flight_num1),
                                                                            (code2, code1, gate2, gate1, flight_num2)]:
                        if origin not in flight_plan: flight_plan[origin] = {}
                        flight_plan[origin][flight_num1] = {
                            "gate": origin_gate,
                            "dest": dest,
                            "dest_gate": dest_gate,
                            "size": size
                        }
                        utils._log(f"{flight_num1} (size {size}): {origin} ({origin_gate}) -> {dest} ({dest_gate})", 1, verbosity)

        # hub-to-hub flights
        flight_nums_h2h = flight_num_generator('h2h')
        utils._log(f"Processing H2H flights ({'existing' if exist_ok else 'non-existing'})", 0, verbosity)
        for code1, code2 in itertools.combinations(config['hubs'], 2):
            if not flight_already_exists(code1, code2):
                size, gate1, gate2 = get_gate(code1, code2, exist_ok=exist_ok)
                if size is None: continue
                try:
                    flight_num1 = next(flight_nums_h2h)
                    flight_num2 = flight_num1 if config['both_dir_same_num'] else next(flight_nums_h2h)
                    flight_nums.append(flight_num1)
                    flight_nums.append(flight_num2)
                except StopIteration:
                    break
                for origin, dest, origin_gate, dest_gate, flight_num in [(code1, code2, gate1, gate2, flight_num1), (code2, code1, gate2, gate1, flight_num2)]:
                    if origin not in flight_plan: flight_plan[origin] = {}
                    flight_plan[origin][flight_num] = {
                        "gate": origin_gate,
                        "dest": dest,
                        "dest_gate": dest_gate,
                        "size": size
                    }
                    utils._log(f"{flight_num1} (size {size}): {origin} ({origin_gate}) -> {dest} ({dest_gate})", 1, verbosity)

        # nonhub-to-nonhub flights
        flight_nums_n2n = flight_num_generator('n2n')
        utils._log(f"Processing N2N flights ({'existing' if exist_ok else 'non-existing'})", 0, verbosity)
        for code1, code2 in itertools.combinations(nonhubs, 2):
            if not flight_already_exists(code1, code2):
                size, gate1, gate2 = get_gate(code1, code2, exist_ok=exist_ok)
                if size is None: continue
                try:
                    flight_num1 = next(flight_nums_n2n)
                    flight_num2 = flight_num1 if config['both_dir_same_num'] else next(flight_nums_n2n)
                except StopIteration:
                    break
                for origin, dest, origin_gate, dest_gate, flight_num in [(code1, code2, gate1, gate2, flight_num1),
                                                                            (code2, code1, gate2, gate1, flight_num2)]:
                    if origin not in flight_plan: flight_plan[origin] = {}
                    flight_plan[origin][flight_num1] = {
                        "gate": origin_gate,
                        "dest": dest,
                        "dest_gate": dest_gate,
                        "size": size
                    }
                    utils._log(f"{flight_num1} (size {size}): {origin} ({origin_gate}) -> {dest} ({dest_gate})", 1, verbosity)

    utils._log("Network planning complete", 0, verbosity)
    
    # == convert to flight list ==
    if output_format == 'json':
        processed_flights = []
        out = []
        for code1, flights in flight_plan.items():
           for num, flightinfo in flights.items():
                if num in processed_flights: continue
                out.append({
                    "number": num,
                    "size": flightinfo['size'],
                    "code1": code1,
                    "gate1": flightinfo['gate'],
                    "code2": flightinfo['dest'],
                    "gate2": flightinfo['dest_gate']
                })
                processed_flights.append(num)
        out.sort(key=lambda x: int(x['number'][2:]))
    
    if verbosity >= 1:
        emptygates = []
        fullgates = []
        for code, gates in gates.items():
            for gateinfo in gates:
                if gateinfo['capacity'] == 6:
                    emptygates.append(f"{code} gate {gateinfo['code']}")
                elif gateinfo['capacity'] == 0:
                    fullgates.append(f"{code} gate {gateinfo['code']}")

        print(term.yellow(f"== Flight network stats =="))
        print(term.yellow(f"Flight:Destination ratio: {len(out) / len(gates)}"))
        print(term.yellow(f"Empty gates: {', '.join(emptygates)}"))
        print(term.yellow(f"Full gates: {', '.join(fullgates)}"))

    return out