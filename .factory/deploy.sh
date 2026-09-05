#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(cd "$(dirname "$0")/.." && pwd)
app_name=sf-client-action-room

# Azure Files is the durable copy, but SQLite takes locks on a local working
# copy. AppState restores at boot and atomically snapshots every successful
# write back to /data. Configure this before the image rollout so both
# revisions use the same storage strategy during cutover.
current_data_dir=$(az containerapp show --resource-group sociobot --name "$app_name" \
  --query "properties.template.containers[0].env[?name=='DATA_DIR'].value | [0]" --output tsv 2>/dev/null || true)
current_persist_dir=$(az containerapp show --resource-group sociobot --name "$app_name" \
  --query "properties.template.containers[0].env[?name=='PERSIST_DIR'].value | [0]" --output tsv 2>/dev/null || true)
if [ "$current_data_dir" != "/tmp/client-action-room" ] || [ "$current_persist_dir" != "/data" ]; then
  az containerapp update --resource-group sociobot --name "$app_name" \
    --set-env-vars DATA_DIR=/tmp/client-action-room PERSIST_DIR=/data --output none
fi

# The fleet helper owns durable-share wiring. WO_DATA_DIR keeps SQLite on the
# product's /data mount and pins the app to one writer without reading secrets.
WO_DATA_DIR=/data /opt/fleet/lib/deploy-container.sh client-action-room "$repo_dir" Dockerfile 8080

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
