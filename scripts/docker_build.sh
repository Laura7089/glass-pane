#!/bin/bash
set -eo pipefail

image_name="$DOCKER_IMAGE"

if [[ $(git rev-parse --abbrev-ref HEAD) -eq "develop" ]]; then
    image_name = "$image_name:develop"
fi

docker build -t "$image_name" .
