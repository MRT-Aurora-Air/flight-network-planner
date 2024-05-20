set -euxo pipefail

./flight-network-planner run config.yml --old out.txt -s > out-new.txt
mv out-new.txt out.txt
./flight-network-planner gate-keys out.txt > gates.txt