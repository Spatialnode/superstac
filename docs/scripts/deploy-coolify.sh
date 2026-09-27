#!/usr/bin/env bash
set -euo pipefail

: "${COOLIFY_DEPLOY_WEBHOOK:?}"
: "${COOLIFY_TOKEN:?}"

# Only the authenticated deploy webhook is called: a deploy-only token is sufficient.
# Coolify pulls the fixed production image tag configured in its dashboard.
# Wait for the rollout in Coolify before requesting another production deployment.
response=$(curl --fail --silent --show-error --proto '=https' \
  --connect-timeout 10 --max-time 60 \
  --header "Authorization: Bearer $COOLIFY_TOKEN" \
  --request POST "$COOLIFY_DEPLOY_WEBHOOK")
deployment=$(jq -er '.deployments[0].deployment_uuid | select(type == "string" and length > 0)' <<< "$response")
[[ "$deployment" =~ ^[a-zA-Z0-9_-]+$ ]] || { echo 'Invalid deployment UUID' >&2; exit 1; }
message="Coolify queued deployment $deployment. Confirm rollout completion in Coolify."
echo "$message"
if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
  echo "$message" >> "$GITHUB_STEP_SUMMARY"
fi
