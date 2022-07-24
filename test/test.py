import mfnp
import json
import yaml

from mfnp.utils import Config, Gate

with open("test/actualgates.txt", "r") as f:
    data = [x.split(" ") for x in f.read().split("\n")] # airport, gate, size

in_: dict[str, list[Gate]] = {}

for gate in data:
    if gate[0] not in in_:
        in_[gate[0]] = []
    in_[gate[0]].append(Gate(
        code=gate[1],
        size=gate[2]
    ))

with open("test/testconfig.yml", "r") as f:
    config = Config.parse_obj(yaml.safe_load(f))
config.gates = in_

out = mfnp.run(config)
with open("test/plan.json", "w") as f:
    json.dump(out, f, indent=2)