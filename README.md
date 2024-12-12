# flight-network-planner
Minecart Rapid Transit Flight Network Planner for airlines

This planner prioritises unique flight routes ie tries its best not to duplicate other airlines' flights :)

Astrella uses this program to generate its flight network and has found that over 90% of flights are unique :eyes:

## Usage
1. Download the planner...
    * ... from the command line: (`<version>` is the version number (with `v`) and `<os>` is one of `windows`, `macos`, `ubuntu`)
      * Windows Powershell: `Invoke-WebRequest -Uri "https://github.com/MRT-Aurora-Air/flight-network-planner/releases/download/<version>/flight-network-planner-<os>" -OutFile "flight-network-planner.exe"`
      * Mac / Linux: `curl "https://github.com/MRT-Aurora-Air/flight-network-planner/releases/download/<version>/flight-network-planner-<os>" -Lo flight-network-planner` (needs curl)
    * ... from Cargo: `cargo install --git https://github.com/MRT-Aurora-Air/flight-network-planner` (omit `./` from this step onwards in this case)
    * ... as an executable (see the releases for downloads for windows, mac and ubuntu) (save it as `flight-network-planner`), you would then have to navigate in the command line to the same directory as where you downloaded the executable to
    * For mac/linux you may have to `chmod +x ./flight-network-planner`, unless you downloaded it via Cargo
2. Run `./flight-network-planner get-config` to get the default configuration file
    * Append `> <config_file_name>` to save the configuration to a file
3. Edit the configuration file for your airline
4. Run `./flight-network-planner run <config_file_name>` to generate the flight plan for your airline
    * Append `-s` to view statistics about the flight plan (you may have to scroll up)
    * Append `-o <old_output_file_name>` if you still have the output of a previous run (to tell the planner to prioritise existing flight routes in your airline over new ones), with `-r` to replace it
    * Appens `> <output_file_name>` to save the output to a file 
5. Profit

## Disclaimer
1. As flight plans depend heavily on other airlines, flight plans can change extremely rapidly over time
2. This program pulls data from [Gatelogue](https://github.com/mrt-map/gatelogue), which means
   * The duplication rate may be higher or lower than actual, depending on whether the MRT Mapping Services have recorded the other airlines' flights
   * You need the internet for this to work
3. There is a 99.9999999% chance something will break while you use the program. I haven't got round to unit-testing the planner thoroughly so there may be bugs lurking everywhere
