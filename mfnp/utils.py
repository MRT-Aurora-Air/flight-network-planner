from __future__ import annotations

import difflib
import os
import time

import requests
import json
from typing import Iterable, TypeVar, TypeAlias, Tuple, Dict, List

import blessed
from pydantic import BaseModel

term = blessed.Terminal()

AirlineName: TypeAlias = str
FlightNumber: TypeAlias = int
AirportCode: TypeAlias = str
_FlightData: TypeAlias = Dict[AirlineName, Dict[FlightNumber, List[AirportCode]]]


class Gate(BaseModel):
    code: str
    size: str
    capacity: int | None = None
    dests: List[AirportCode] | None = None
    score: int | None = None


class Config(BaseModel):
    airline_name: AirlineName
    airline_code: str
    ignored_airlines: List[AirlineName]
    hubs: List[AirportCode]
    hub_threshold: int | None = None
    range_h2h: List[Tuple[FlightNumber, FlightNumber]]
    range_n2n: List[Tuple[FlightNumber, FlightNumber]]
    range_h2n: Dict[AirportCode, List[Tuple[FlightNumber, FlightNumber]]]
    both_dir_same_num: bool
    gate_json: str | None = None
    gates: Dict[AirportCode, List[Gate]]
    hard_max: int
    max_h2n: int
    max_n2n: int


class FlightRoute(BaseModel):
    code: AirportCode
    orig: AirportCode
    orig_gate: str
    dest: AirportCode
    dest_gate: str
    size: str


def _warn(content: str, **kwargs):
    print(term.yellow(content), **kwargs, flush=True)


def _log(content: str, **kwargs):
    print(term.white(content), **kwargs, flush=True)


def _debug(content: str, **kwargs):
    print(term.bright_black(content), **kwargs, flush=True)


_T = TypeVar("_T")


def _gcm(input_: _T, options: Iterable[_T]) -> str:
    return ", ".join(difflib.get_close_matches(input_, options))


def cache_flight_data(data: _FlightData, airport_codes: List[AirportCode]):
    try:
        os.mkdir(os.path.dirname(__file__) + "/.cache")
    except FileExistsError:
        pass
    with open(os.path.dirname(__file__) + "/.cache/flights.json", "w+") as f:
        json.dump(
            {"data": data, "airport_codes": airport_codes, "timestamp": time.time()},
            f,
            indent=2,
        )


def get_flight_data(nocache: bool = False) -> Tuple[_FlightData, List[str]]:
    # check cache
    try:
        with open(os.path.dirname(__file__) + "/.cache/flights.json", "r") as f:
            j = json.load(f)
            if time.time() - j["timestamp"] <= 60 and not nocache:
                _log("Retrieved raw flight data from cache")
                return j["data"], j["airport_codes"]
    except FileNotFoundError:
        pass

    _log("Retrieving raw flight data... ", end="")
    raw_airline_data = requests.get(
        "https://sheets.googleapis.com/v4/spreadsheets/1wzvmXHQZ7ee7roIvIrJhkP6oCegnB8-nefWpd8ckqps/values/Airline+Class+Distribution?key=AIzaSyCCRZqIOAVfwBNUofWbrkz0q5z4FUaCUyE"
    )
    raw_airline_data = json.loads(raw_airline_data.text)["values"]
    raw_seaplane_data = requests.get(
        "https://sheets.googleapis.com/v4/spreadsheets/1wzvmXHQZ7ee7roIvIrJhkP6oCegnB8-nefWpd8ckqps/values/Seaplane+Class+Distribution?key=AIzaSyCCRZqIOAVfwBNUofWbrkz0q5z4FUaCUyE"
    )
    raw_seaplane_data = json.loads(raw_seaplane_data.text)["values"]
    _log("retrieved")

    data: _FlightData = {}
    airport_codes = []
    num_of_empty_codes = 0
    for raw_data in [raw_airline_data, raw_seaplane_data]:
        airlines = raw_data[1]  # get first row, it contains names of all airlines
        cursor = 2
        while (
            (not raw_data[cursor][0].startswith("Total Flights"))
            if len(raw_data[cursor]) != 0
            else True
        ):  # iterate through all airports
            if (
                raw_data[cursor] == [] or len(raw_data[cursor]) <= 4
            ):  # if row is empty or has no flights to it, continue
                cursor += 1
                continue
            airport_code = raw_data[cursor][1]
            if airport_code.strip() == "":
                airport_code = "??#" + str(num_of_empty_codes)
                num_of_empty_codes += 1
            airport_codes.append(airport_code)
            _debug(f"Processing {airport_code}")
            for index, airport_flights in enumerate(
                raw_data[cursor]
            ):  # iterate through airlines
                if (
                    airlines[index] != "" and airport_flights != ""
                ):  # if it isn't empty, there are flights
                    airline_name = airlines[index]
                    airline_flights = list(
                        map(lambda num: num.strip(), airport_flights.split(","))
                    )
                    if airline_name not in data:
                        data[airline_name] = {}
                    for flight in airline_flights:
                        if flight not in data[airline_name]:
                            data[airline_name][flight] = []
                        data[airline_name][flight].append(airport_code)
            cursor += 1

    _log("Flight data processed")
    cache_flight_data(data, airport_codes)
    return data, airport_codes
