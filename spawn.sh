#!/bin/bash

if [ $# -eq 0 ]; then
  echo "Usage: $0 <N>"
  exit 1
fi

N=$1
START_PORT=50001

cleanup() {
  pkill -P $$
  exit 0
}

trap cleanup EXIT

for ((i=0; i<N; i++))
do
  PORT=$((START_PORT + i))
  sleep 0.2
  cargo run --bin=dht $PORT &
done

wait