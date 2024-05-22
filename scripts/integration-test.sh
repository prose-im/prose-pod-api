#!/bin/bash

# Redirect users to <CONTRIBUTING.md> if env vars are not set.
# NOTE: We still use `${X:?}`, this is just to improve the dev UX.
# COPYRIGHT: <https://stackoverflow.com/a/65396324/10967642>.
test-env-vars() {
  var_names=("$@");
  for var_name in "${var_names[@]}"; do
    [ -z "${!var_name}" ] && echo "${var_name} isn't set. Check <CONTRIBUTING.md> for more information." && var_unset=true;
  done
  [ -n "$var_unset" ] && exit 1;
  return 0;
}
test-env-vars \
  PROSE_POD_API_DIR \
  PROSE_POD_SYSTEM_DIR;

STEPCI_DIR="${PROSE_POD_API_DIR:?}"/tests/integration/step-ci;
ENV_FILE="${PROSE_POD_API_DIR:?}"/tests/integration/in-memory.env;

clean-prosody() {
  rm -rf "${PROSE_POD_SYSTEM_DIR:?}"/server/var/lib/prosody/*%2e*;
}
start() {
  START_TIME=$(date +%s);
  clean-prosody;
  ENV_FILE="${ENV_FILE:?}" docker compose -f "${PROSE_POD_SYSTEM_DIR:?}"/compose.yaml up --detach;
}
stop() {
  docker compose -f "${PROSE_POD_SYSTEM_DIR:?}"/compose.yaml stop;
}
abort() {
  stop;
  local current_time=$(date +%s);
  local elapsed_time=$((current_time - ${START_TIME:?}));
  docker compose -f "${PROSE_POD_SYSTEM_DIR:?}"/compose.yaml logs --since "${elapsed_time}s" server api;
  exit 1;
}

stepci_run() {
  # local jwt_signing_key=$(source "${ENV_FILE:?}" && echo "${JWT_SIGNING_KEY:?}");
  # local admin_jwt=$(jwt encode --secret="${jwt_signing_key}" '{"jid": "example@prose.org"}');
  start && stepci run "${STEPCI_DIR:?}/${1:?}.yaml" && stop || abort;
}

stepci_run init;
stepci_run global;
