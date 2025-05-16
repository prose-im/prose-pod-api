# NOTE: The `: ${VAR_NAME:=value}` syntax initializes a variable only if it's unset or null.
#   It avoids resetting a variable when sourcing this file after the variable was overriden.

: ${PROSE_POD_API_IMAGE_TAG:=0.14.0}
: ${PROSE_POD_SERVER_IMAGE_TAG:=0.3.19}
LOCAL_RUN_DIR="${REPOSITORY_ROOT:?}"/local-run
: ${COMPOSE_FILE:="${LOCAL_RUN_DIR:?}"/compose.yaml}
DEFAULT_SCENARIO_NAME=default
: ${SCENARIOS_DIR:="${LOCAL_RUN_DIR:?}"/scenarios}
EPHEMERAL_SCENARIOS_FILE="${SCENARIOS_DIR:?}"/ephemeral-scenarios.txt

source "${SCRIPTS_ROOT:?}"/constants.sh
source "${SCRIPTS_ROOT:?}"/image-names.sh
