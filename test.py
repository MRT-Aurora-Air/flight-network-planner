import mfnp
import json
import yaml

from mfnp.utils import Config, Gate

with open("actualgates.json", "r") as f:
    data = json.load(f)

in_: dict[str, list[Gate]] = {}

for gate in data:
    if gate['code'] not in in_:
        in_[gate['code']] = []
    in_[gate['code']].append(Gate(
        code=gate['name'],
        size=gate['size']
    ))

with open("testconfig.yml", "r") as f:
    config = Config.parse_obj(yaml.safe_load(f))
config.gates = in_

out = mfnp.run(config)
with open("plan.json", "w") as f:
    json.dump(out, f, indent=2)