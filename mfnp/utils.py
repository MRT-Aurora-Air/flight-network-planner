import difflib
import os
import sys
import time

import requests
import json
from typing import List, Tuple, Iterable, Any, Dict

import blessed

term = blessed.Terminal()

def _warn(content: str, nowarn: bool, **kwargs):
    if not nowarn:
        print(term.yellow(content), **kwargs, flush=True)

def _log(content: str, level: int, verbosity: int, **kwargs):
    if verbosity >= level:
        colour = term.white if level == 0 else term.bright_black if level == 1 else term.white
        print(colour(content), **kwargs, flush=True)

def _gcm(input_: Any, options: Iterable) -> str:
    return ', '.join(difflib.get_close_matches(input_, options))

def cache_flight_data(data: Dict[str, Dict[str, List[str]]], airport_codes: List[str]):
    try:
        os.mkdir(os.path.dirname(__file__)+'/.cache')
    except FileExistsError:
        pass
    with open(os.path.dirname(__file__)+"/.cache/flights.json", "w+") as f:
        json.dump({
            "data": data,
            "airport_codes": airport_codes,
            "timestamp": time.time()
        }, f, indent=2)
        f.close()

def get_flight_data(verbosity: int=0, nocache: bool=False) -> Tuple[Dict[str, Dict[str, List[str]]], List[str]]:
    # check cache
    try:
        with open(os.path.dirname(__file__) + "/.cache/flights.json", "r") as f:
            j = json.load(f)
            if time.time() - j['timestamp'] <= 60 and not nocache:
                f.close()
                _log("Retrieved raw flight data from cache", 0, verbosity)
                return j['data'], j['airport_codes']
            f.close()
    except FileNotFoundError:
        pass
    _log("Retrieving raw flight data... ", 0, verbosity, end="")
    raw_airline_data = requests.get("https://sheets.googleapis.com/v4/spreadsheets/1wzvmXHQZ7ee7roIvIrJhkP6oCegnB8-nefWpd8ckqps/values/Airline+Class+Distribution?key=AIzaSyCCRZqIOAVfwBNUofWbrkz0q5z4FUaCUyE")
    raw_airline_data = json.loads(raw_airline_data.text)['values']
    raw_seaplane_data = requests.get("https://sheets.googleapis.com/v4/spreadsheets/1wzvmXHQZ7ee7roIvIrJhkP6oCegnB8-nefWpd8ckqps/values/Seaplane+Class+Distribution?key=AIzaSyCCRZqIOAVfwBNUofWbrkz0q5z4FUaCUyE")
    raw_seaplane_data = json.loads(raw_seaplane_data.text)['values']
    _log("retrieved", 0, verbosity)

    data = {} # {airline_name: {flight_num: [ABC, DEF]}}
    airport_codes = []
    num_of_empty_codes = 0
    for raw_data in [raw_airline_data, raw_seaplane_data]:
        airlines = raw_data[1] # get first row, it contains names of all airlines
        cursor = 2
        while (not raw_data[cursor][0].startswith("Total Flights")) if len(raw_data[cursor]) != 0 else True: # iterate through all airports
            if raw_data[cursor] == [] or len(raw_data[cursor]) <= 4:  # if row is empty or has no flights to it, continue
                cursor += 1
                continue
            airport_code = raw_data[cursor][1]
            if airport_code.strip() == "":
                airport_code = "??#"+str(num_of_empty_codes)
                num_of_empty_codes += 1
            airport_codes.append(airport_code)
            _log(f"Processing {airport_code}", 1, verbosity)
            for index, airport_flights in enumerate(raw_data[cursor]): # iterate through airlines
                if airlines[index] != "" and airport_flights != "": # if it isnt empty, there are flights
                    airline_name = airlines[index]
                    airline_flights = list(map(lambda num: num.strip(), airport_flights.split(',')))
                    if airline_name not in data: data[airline_name] = {}
                    for flight in airline_flights: 
                        if flight not in data[airline_name]: data[airline_name][flight] = []
                        data[airline_name][flight].append(airport_code)
            cursor += 1
    _log("Flight data processed", 0, verbosity)
    cache_flight_data(data, airport_codes)
    return data, airport_codes