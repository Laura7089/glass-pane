#!/bin/bash
set -eo pipefail

image_name="$1"
username="$2"
password="$3"

echo "$password" | docker login -u "$username"

if [[ $(git rev-parse --abbrev-ref HEAD) -eq "develop" ]]; then
    image_name = "$image_name:develop"
fi

docker push "$image_name"
