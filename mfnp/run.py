import itertools
from typing import Optional, Tuple

import blessed
import json
term = blessed.Terminal()

import utils

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

    # throw out ignored airlines
    for airline in config['ignored_airlines']:
        if airline in data: del data[airline]
        else: utils._warn(f"Airline `{airline}` doesn't exist, maybe you mean: {utils._gcm(airline, data.keys())}", nowarn)

    # double-check all airport codes
    for code in config['gates'].keys():
        if code not in airports:
            utils._warn(f"Airport `{code}` doesn't exist, maybe you mean: {utils._gcm(code, airports)}", nowarn)
    for code in config['hubs']:
        if code not in config['gates'].keys():
            utils._warn(f"Airport `{code}` has no gates but is stated as a hub, maybe you mean: {utils._gcm(code, config['gates'].keys())}", nowarn)
    config['hubs'] = list(filter(lambda c: c in config['gates'].keys(), config['hubs']))

    # TODO ensure range_h2n for hubs

    # == Plan the flights ==

    flight_plan = {} # {airport_code: {flight_num: {gate: XXX, dest: XXX, dest_gate: XXX, size: X}}}
    nonhubs = [g for g in config['gates'].keys() if g not in config['hubs']]
    gates = config['gates'].copy()
    for gate_set in gates.values():
        for gate in gate_set:
            gate['capacity'] = 6

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
            yield config['airline_code']+str(num)
        utils._warn(f"Not enough flight numbers for {mode} flights" + (f" from {code_}" if code_ else ""), nowarn)
        # TODO ensure no overlap of flight nums

    def get_gate(c1: str, c2: str) -> Tuple[Optional[str], Optional[str], Optional[str]]:
        for g1 in gates[c1]:
            for g2 in gates[c2]:
                if g1['size'] == g2['size'] and g1['capacity'] > 0 and g2['capacity'] > 0:
                    g1['capacity'] -= 1
                    g2['capacity'] -= 1
                    return g1['size'], g1['code'], g2['code']
        return None, None, None

    # hub-to-hub flights (not existing)
    flight_nums_h2h = flight_num_generator('h2h')
    for code1, code2 in itertools.combinations(config['hubs'], 2):
        if not flight_already_exists(code1, code2):
            size, gate1, gate2 = get_gate(code1, code2)
            if size is None: continue
            try:
                flight_num1 = next(flight_nums_h2h)
                flight_num2 = flight_num1 if config['both_dir_same_num'] else next(flight_nums_h2h)
            except StopIteration:
                break
            for origin, dest, origin_gate, dest_gate, flight_num in [(code1, code2, gate1, gate2, flight_num1), (code2, code1, gate2, gate1, flight_num2)]:
                if origin not in flight_plan: flight_plan[origin] = {}
                flight_plan[origin][flight_num1] = {
                    "gate": origin_gate,
                    "dest": dest,
                    "dest_gate": dest_gate,
                    "size": size
                }

    # hub-to-nonhub flights
    for code1 in config['hubs']:
        flight_nums_h2n = flight_num_generator('h2n', code_=code1)
        for code2 in nonhubs:
            if not flight_already_exists(code1, code2):
                size, gate1, gate2 = get_gate(code1, code2)
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

    # huh-to-hub flights (existing)

    # nonhub-to-nonhub flights

    print(json.dumps(flight_plan, indent=2))