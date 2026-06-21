#!/bin/bash
trap 'podman compose down; yes | podman system prune --build; exit' INT

podman compose build
podman compose up -d
podman compose logs -f &

read -r -d '' _ </dev/tty
