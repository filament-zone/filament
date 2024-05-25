#!/bin/bash

export DOCKER_CLI_HINTS=false

command -v docker >/dev/null 2>&1 || { echo >&2 "docker not found"; exit 1; }

if ! docker pull neutronorg/neutron:v3.0.5 ; then echo >&2 "failed to pull image"; exit 1; fi

docker rm -f neutron || true
docker run --rm --name neutron -d -v $(dirname "$0"):/contracts -p 1317:1317 -p 26657:26657 -p 26656:26656 -p 16657:16657 -p 8090:9090 -e RUN_BACKGROUND=0 neutron-node
