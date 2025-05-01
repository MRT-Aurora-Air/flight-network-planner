# flight-network-planner
[Minecart Rapid Transit](https://minecartrapidtransit.net) Flight Network Planner for airlines

This planner prioritises unique flight routes and tries its best not to duplicate other airlines' flights :)

[Astrella](https://wiki.minecartrapidtransit.net/index.php/Astrella) uses this program to generate its flight network and has found that over 90% of flights are unique :eyes:

## Usage
1. Run `cargo install flight-network-planner` to download
    * `cargo-binstall` is supported: `cargo binstall flight-network-planner`
    * If cargo is not available, prebuilt executables are located in GitHub releases
2. Run `flight-network-planner get-config` to get the default configuration file
    * Append `> <config_file_name>` to save the configuration to a file
3. Edit the configuration file for your airline
4. Run `flight-network-planner run <config_file_name>` to generate the flight plan for your airline
    * Append `-s` to view statistics about the flight plan (you may have to scroll up)
    * Append `-o <old_output_file_name>` if you still have the output of a previous run (to tell the planner to preserve flight numbers), with `-r` to replace it
    * Appens `> <output_file_name>` to save the output to a file 
5. Profit

## Disclaimer
1. As flight plans depend heavily on other airlines, flight plans can change extremely rapidly over time
2. This program pulls data from [Gatelogue](https://github.com/mrt-map/gatelogue), which means
   * The duplication rate may be higher or lower than its actual value, depending on whether the MRT Mapping Services have recorded the other airlines' flights
   * You need the internet for this to work
3. There is a 99.9999999% chance something will break while you use the program. I haven't got round to unit-testing the planner thoroughly so there may be bugs lurking everywhere
