# NOTE: The `: ${VAR_NAME:=value}` syntax initializes a variable only if it's unset or null.
#   It avoids resetting a variable when sourcing this file after the variable was overriden.

: ${PROSE_DOCKER_ORG:=proseim}
: ${LATEST_DOCKER_TAG:=latest}
# Tag used to reference images built locally (e.g. uncommitted code).
: ${LOCAL_DOCKER_TAG:=local}
: ${DEFAULT_DOCKER_TAG:="${LATEST_DOCKER_TAG:?}"}
