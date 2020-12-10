#!/bin/bash
set -eo pipefail

image_name="$DOCKER_IMAGE"
username="$DOCKER_USERNAME"
password="$DOCKER_PASSWORD"

echo "$password" | docker login -u "$username"

if [[ $(git rev-parse --abbrev-ref HEAD) -eq "develop" ]]; then
    image_name = "$image_name:develop"
fi

docker push "$image_name"
