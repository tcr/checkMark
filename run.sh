#!/bin/bash

docker run --rm -it \
  --pid host \
  --mount type=bind,source="$(pwd)",target=/app \
  --mount type=bind,source="$(realpath rmapi)",target=/root/.rmapi \
  --mount type=bind,source="$(pwd)/target",target=/app/target \
  --mount type=bind,source="$(pwd)/data",target=/app/data \
  -p 8080:8080 \
  $(docker build -q .) "$@"
