import os
import blessed
term = blessed.Terminal()

def get_config(name: str, force: bool=False) -> None:
    """Creates a configuration file for the flight network planner."""
    with open(os.path.dirname(__file__)+"/defaultconfig.yaml", "r") as f:
        data = f.read()
        f.close()
    with open(name+".yaml", "w") as f:
        if f.read() != data and not force:
            print(term.bright_red(f"{name}.yaml has been edited; use `-f` to overwrite"))
            return
        f.write(data)
        f.close()