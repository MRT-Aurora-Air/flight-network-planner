import mfnp
import json
import yaml

with open("actualgates.json", "r") as f:
    data = json.load(f)

in_ = {}

for gate in data:
    if gate['code'] not in in_:
        in_[gate['code']] = []
    in_[gate['code']].append({
        "code": gate['name'],
        "size": gate['size']
    })

with open("testconfig.yml", "r") as f:
    config = yaml.safe_load(f)
config['gates'] = in_

out = mfnp.run(config, verbosity=1)
with open("plan.json", "w") as f:
    json.dump(out, f, indent=2)
    f.close()