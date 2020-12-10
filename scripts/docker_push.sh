#!/bin/bash
set -eo pipefail

echo "$DOCKER_PASSWORD" | docker login -u "$DOCKER_USERNAME"

docker push "$DOCKER_IMAGE"
