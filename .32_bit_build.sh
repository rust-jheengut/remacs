#!/bin/bash

# Run a 32 bit build inside a docker container on Travis.

set -e
set -x

apt-get -qq update
apt-get install build-essential autoconf

cd /remacs

./autogen.sh
./configure
make -j 3
