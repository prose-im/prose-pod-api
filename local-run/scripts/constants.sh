# NOTE: The `: ${VAR_NAME:=value}` syntax initializes a variable only if it's unset or null.
#   It avoids resetting a variable when sourcing this file after the variable was overriden.

: ${PROSE_POD_API_IMAGE_TAG:=0.6.0}
: ${PROSE_POD_SERVER_IMAGE_TAG:=0.3.4}
LOCAL_RUN_DIR="${PROSE_POD_API_DIR:?}"/local-run
: ${COMPOSE_FILE:="${LOCAL_RUN_DIR:?}"/compose.yaml}
DEFAULT_SCENARIO_NAME=default
SCENARIOS_DIR="${LOCAL_RUN_DIR:?}"/scenarios

source "${SCRIPTS_ROOT:?}"/constants.sh
source "${SCRIPTS_ROOT:?}"/image-names.sh
