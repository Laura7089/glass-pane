#!/bin/bash
set -eo pipefail

docker login -u "$DOCKER_USERNAME" -p "$DOCKER_PASSWORD"
docker push "$DOCKER_IMAGE"
