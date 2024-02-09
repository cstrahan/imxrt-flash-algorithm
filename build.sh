#!/bin/bash

set -euo pipefail

ELF=target/thumbv7em-none-eabihf/release/imxrt-flash-algorithm
YAML=MIMXRT1060.yaml
STACK_SIZE=2048

cargo build --release && \
  target-gen elf -u "$ELF" "$YAML" && \
  # add stack_size after instructions
  perl -pi -e '/instructions:/ and $_.="  stack_size: '$STACK_SIZE'\n"' MIMXRT1060.yaml
    
#sed -e 's/algorithm-test # \(.*\)$/\1/' template.yaml > "$out_yaml"
