#!/bin/bash

# Run cargo test to generate all TypeScript bindings, endpoint files, and API wrappers.
rm -f ./bindings/*.ts; cargo test export_

# Run prettier twice: the @trivago/prettier-plugin-sort-imports plugin can
# reorder `export type { X } from '...'` re-exports on the first pass, so a
# second pass is needed to reach a stable state before committing.
#npx prettier --write ./bindings/* && npx prettier --write ./bindings/*
