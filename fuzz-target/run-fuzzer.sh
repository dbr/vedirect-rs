#!/bin/sh

cargo afl build && env AFL_SKIP_CPUFREQ=1 cargo afl fuzz -i ./in -o ./out ./target/debug/fuzz-target
