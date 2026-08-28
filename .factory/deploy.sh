#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(cd "$(dirname "$0")/.." && pwd)
/opt/fleet/lib/deploy-container.sh client-action-room "$repo_dir" Dockerfile 8080

# Keep the SQLite file on the factory's Azure Files account and run one writer.
# This preserves rooms across restarts and prevents split-brain replicas.
storage_key=$(az storage account keys list --resource-group sociobot --account-name sociobotblob --query '[0].value' --output tsv)
az storage share create --name client-action-room --account-name sociobotblob --account-key "$storage_key" --quota 5 --output none
az containerapp env storage set \
  --resource-group sociobot \
  --name factory-env \
  --storage-name client-action-room-data \
  --azure-file-account-name sociobotblob \
  --azure-file-account-key "$storage_key" \
  --azure-file-share-name client-action-room \
  --access-mode ReadWrite \
  --output none

image=$(az containerapp show --resource-group sociobot --name sf-client-action-room --query 'properties.template.containers[0].image' --output tsv)
deploy_yaml=$(mktemp)
trap 'rm -f "$deploy_yaml"' EXIT
sed "s|IMAGE_PLACEHOLDER|$image|" "$repo_dir/.factory/containerapp-volume.yaml" > "$deploy_yaml"
az containerapp update \
  --resource-group sociobot \
  --name sf-client-action-room \
  --yaml "$deploy_yaml" \
  --output none

actual=$(az containerapp show --resource-group sociobot --name sf-client-action-room \
  --query 'properties.template.scale.maxReplicas' --output tsv)
test "$actual" = "1"

latest=$(az containerapp show --resource-group sociobot --name sf-client-action-room --query 'properties.latestRevisionName' --output tsv)
for _ in $(seq 1 60); do
  health=$(az containerapp revision show --resource-group sociobot --name sf-client-action-room --revision "$latest" --query 'properties.healthState' --output tsv)
  [ "$health" = "Healthy" ] && break
  sleep 5
done
test "$health" = "Healthy"
while IFS= read -r revision; do
  if [ -n "$revision" ] && [ "$revision" != "$latest" ]; then
    az containerapp revision deactivate --resource-group sociobot --name sf-client-action-room --revision "$revision" --output none
  fi
done < <(az containerapp revision list --resource-group sociobot --name sf-client-action-room --query '[?properties.active].name' --output tsv)
