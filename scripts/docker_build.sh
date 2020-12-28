#!/bin/bash
set -eo pipefail

docker build -t "${DOCKER_IMAGE}" .
docker build -t "${DOCKER_IMAGE}-nocommand" --build-arg "CARGO_FLAGS=''" .
