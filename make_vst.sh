#!/bin/bash

cargo build --release --target x86_64-pc-windows-gnu
mkdir -p target/vst3;
cp target/x86_64-pc-windows-gnu/release/stoner.dll target/vst3/Stoner.vst3;