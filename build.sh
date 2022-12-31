#!/bin/bash

cargo build --release
cp target/release/slidingpuzzle out
cp -r resources out/
