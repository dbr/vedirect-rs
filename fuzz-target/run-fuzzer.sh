#!/bin/sh

cargo afl build --release && env AFL_SKIP_CPUFREQ=1 cargo afl fuzz -i ./in -o ./out ./target/release/fuzz-target
