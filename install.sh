#!/bin/bash
set -euxo pipefail

curl "https://github.com/MRT-Aurora-Air/flight-network-planner/releases/download/v1.2.3/flight-network-planner-ubuntu" -Lo flight-network-planner
chmod +x flight-network-planner
