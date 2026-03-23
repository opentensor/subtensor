#!/bin/bash

docker build --build-arg RUSTC_VERSION="1.89.0" -t srtool https://github.com/paritytech/srtool.git#refs/tags/v0.18.3
