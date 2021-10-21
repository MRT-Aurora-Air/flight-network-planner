def get_config(name: str) -> None:
    """Creates a configuration file for the flight network planner."""
    with open("defaultconfig.yaml", "r") as f:
        data = f.read()
        f.close()
    with open(name+".yaml", "r+") as f:
        f.write(data)
        f.close()
    pass