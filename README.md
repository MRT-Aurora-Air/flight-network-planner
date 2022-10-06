# flight-network-planner
Minecart Rapid Transit Flight Network Planner for airlines

This planner prioritises unique flight routes ie tries its best not to duplicate other airlines' flights :)

Astrella uses this program to generate its flight network and has found that over 90% of flights are unique :eyes:

## Usage
1. Download the planner as an executable (see the releases for downloads for windows, mac and ubuntu)
2. Navigate in the command line to the same directory as where you downloaded the executable to (you may also want to save it as `flight-network-planner`)
3. Run `flight-network-planner get-config` to get the default configuration file
    * Append `-o <config_file_name>` to save the configuration as a specific name
4. Edit the configuration file for your airline
5. Run `flight-network-planner run <config_file_name>` to generate the flight plan for your airline
    * Append `-s` to view statistics about the flight plan (you may have to scroll up)
    * Append `-o <output_file_name>` to output the flight plan to a file
    * If you have an older plan that had already been generated before, append `--old <old_output_file_name>` to tell the planner to prioritise existing flight routes in your airline over new ones
6. Profit

## Disclaimer
1. As flight plans depend heavily on other airlines, flight plans can change extremely rapidly over time
2. This program pulls data from MRT Transit, which means
   * The duplication rate may be higher or lower than actual, depending on whether MRT Transit has recorded the other airlines' flights
   * You need the internet for this to work
3. There is a 99.9999999% chance something will break while you use the program. I haven't got round to unit-testing the planner thoroughly so there may be bugs lurking everywhere
