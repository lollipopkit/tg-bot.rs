#!/bin/sh

set -e

cargo build --release

docker compose down --remove-orphans

image_id=$(docker images -q tg-bot-rs)
if [ ! -z "$image_id" ]; then
  docker image rm -f $image_id
fi

docker build -t tg-bot-rs:latest --rm --force-rm .
docker compose up -d
