#!/bin/bash
ROOT=$(cd $(dirname $0); pwd)

# https://betterprogramming.pub/best-practices-for-bash-scripts-17229889774d
set -o errexit
set -o nounset
set -o pipefail

# Start Vite in watch mode in background
cd ${ROOT}/../renderer
pnpm run dev --logLevel silent >/dev/null 2>&1 &
VITE_PID=$!

# Trap to kill Vite on exit
trap "kill $VITE_PID 2>/dev/null || true" EXIT
# Start Dioxus dev server
cd ${ROOT}/../desktop
dx serve
# Kill Vite when dx serve exits
kill $VITE_PID 2>/dev/null || true
