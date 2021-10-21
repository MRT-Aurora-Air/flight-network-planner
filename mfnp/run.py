import blessed
import json
import difflib
term = blessed.Terminal()

import utils

def run(config: dict, format: str="json", verbosity: int=0, nowarn: bool=False):
    ## == Preparatory stuff ==
    # fill in defaults
    utils._log("Filling in defaults", 0, verbosity)
    if config['ignored_airlines'] == []: config['ignored_airlines'].append(config['airline_name'])
    if config['gate_json']:
        with open(config['gate_json'], 'r') as f:
            config['gates'] = json.load(f)
    if config['hubs'] == []:
        for airport, gates in config['gates'].items():
            if len(gates) >= config['hub_threshold']: config['hubs'].append(airport)
    #print(config)

    # get data from the transit sheet
    data, airports = utils.get_flight_data(verbosity=verbosity)

    # throw out ignored airlines
    for airline in config['ignored_airlines']:
        if airline in data: del data[airline]
        else: utils._warn(f"Airline `{airline}` doesn't exist, maybe you mean: {', '.join(difflib.get_close_matches(airline, data.keys()))}", nowarn)

    # double-check all airport codes
    for code in config['gates'].keys():
        if code not in airports:
            utils._warn(f"Airport `{code}` doesn't exist, maybe you mean: {','.join(difflib.get_close_matches(code, airports))}", nowarn)
    for code in config['hubs']:
        if code not in config['gates'].keys():
            utils._warn(f"Airport `{code}` has no gates but is stated as a hub, maybe you mean: {','.join(difflib.get_close_matches(code, config['gates'].keys()))}", nowarn)
    config['hubs'] = list(filter(lambda code: code in config['gates'].keys(), config['hubs']))

    # == hub-to-hub flights ==


    # === hub-to-nonhub flights ==

    # === nonhub-to-nonhub flights ==