#!/bin/bash
set -eo pipefail

docker build -t "$DOCKER_IMAGE" .
