#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(cd "$(dirname "$0")/.." && pwd)
app_name=sf-client-action-room

ensure_snapshot_environment() {
  current_data_dir=$(az containerapp show --resource-group sociobot --name "$app_name" \
    --query "properties.template.containers[0].env[?name=='DATA_DIR'].value | [0]" --output tsv)
  current_persist_dir=$(az containerapp show --resource-group sociobot --name "$app_name" \
    --query "properties.template.containers[0].env[?name=='PERSIST_DIR'].value | [0]" --output tsv)
  if [ "$current_data_dir" != "/tmp/client-action-room" ] || [ "$current_persist_dir" != "/data" ]; then
    az containerapp update --resource-group sociobot --name "$app_name" \
      --set-env-vars DATA_DIR=/tmp/client-action-room PERSIST_DIR=/data --output none
  fi
}

# Azure Files is the durable copy, but SQLite takes locks on a local working
# copy. AppState restores at boot and atomically snapshots every successful
# write back to /data. Configure an existing app before the image rollout so
# both revisions use the same storage strategy during cutover.
if az containerapp show --resource-group sociobot --name "$app_name" --query name --output tsv >/dev/null 2>&1; then
  ensure_snapshot_environment
fi

# The fleet helper owns durable-share wiring. WO_DATA_DIR mounts the product's
# durable /data share and pins the app to one writer without reading secrets.
WO_DATA_DIR=/data /opt/fleet/lib/deploy-container.sh client-action-room "$repo_dir" Dockerfile 8080

# A first deployment did not have an app to configure before the helper ran.
# Apply the snapshot environment now; this creates the revision checked below.
ensure_snapshot_environment

# Revision mode can briefly leave an older 0%-traffic process alive. Stop those
# product revisions after the new one is healthy so only one SQLite writer can
# publish snapshots to /data.
latest=$(az containerapp show --resource-group sociobot --name "$app_name" \
  --query 'properties.latestRevisionName' --output tsv)
for _ in $(seq 1 60); do
  health=$(az containerapp revision show --resource-group sociobot --name "$app_name" \
    --revision "$latest" --query 'properties.healthState' --output tsv)
  [ "$health" = "Healthy" ] && break
  sleep 5
done
test "$health" = "Healthy"
while IFS= read -r revision; do
  if [ -n "$revision" ] && [ "$revision" != "$latest" ]; then
    az containerapp revision deactivate --resource-group sociobot --name "$app_name" \
      --revision "$revision" --output none
  fi
done < <(az containerapp revision list --resource-group sociobot --name "$app_name" \
  --query '[?properties.active].name' --output tsv)
