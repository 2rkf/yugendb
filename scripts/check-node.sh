#!/usr/bin/env bash
set -euo pipefail

if [ -f pnpm-lock.yaml ]; then
  pnpm install --frozen-lockfile
else
  pnpm install
fi

pnpm typecheck
pnpm test
pnpm build
