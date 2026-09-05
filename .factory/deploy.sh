#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(cd "$(dirname "$0")/.." && pwd)

# The fleet helper owns durable-share wiring. WO_DATA_DIR keeps SQLite on the
# product's /data mount and pins the app to one writer without reading secrets.
WO_DATA_DIR=/data /opt/fleet/lib/deploy-container.sh client-action-room "$repo_dir" Dockerfile 8080
