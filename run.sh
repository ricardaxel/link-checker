#!/usr/bin/env bash

set -euo pipefail

target_dir=$1 

docker build . --tag "link_checker_image:latest" --quiet 
docker run -it -v "$target_dir":"$target_dir" link_checker_image:latest "$target_dir"
