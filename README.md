# Zappi Collector
This is a toy Rust program which hits the API for a Myenergi Zappi electric vehicle charger and retrieves the power figures for the house consumption, solar generation and EV charging.

Currently it is expected that this will be run each day and will fetch the previous days data (which is returned as JSON with per-minute resolution i.e. 1440 data points).

TODO: accept a command line switch to specify another date for which to fetch data instead of yesterday.