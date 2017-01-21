#!/bin/bash

# Run a 32 bit build inside a docker container on Travis.

set -e
set -x

# The chroot path is here.
cd /remacs

./configure
make -j 3
