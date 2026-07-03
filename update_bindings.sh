#!/bin/bash

# Run cargo test to generate all TypeScript bindings, endpoint files, and API wrappers.
rm -rf ./bindings/endpoints/ ./bindings/api/ ./bindings/models/
cargo test export_

# Run prettier twice: the @trivago/prettier-plugin-sort-imports plugin can
# reorder `export type { X } from '...'` re-exports on the first pass, so a
# second pass is needed to reach a stable state before committing.
cd ../qcext-client/
npx prettier --write ./src/bindings/* && npx prettier --write ./src/bindings/*
