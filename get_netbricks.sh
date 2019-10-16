#!/bin/sh

set -e

# Initialize dpdk module
git submodule init
git submodule update --recursive

