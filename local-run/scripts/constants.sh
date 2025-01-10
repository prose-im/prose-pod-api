# NOTE: The `: ${VAR_NAME:=value}` syntax initializes a variable only if it's unset or null.
#   It avoids resetting a variable when sourcing this file after the variable was overriden.

: ${LOCAL_RUN_DIR:="${PROSE_POD_API_DIR:?}"/local-run}
: ${PROSE_POD_API_IMAGE_TAG:=0.6.0}
: ${PROSE_POD_SERVER_IMAGE_TAG:=0.3.4}
: ${DATABASE_NAME:="local-run"}
: ${DATABASE_PATH:="${LOCAL_RUN_DIR:?}"/"${DATABASE_NAME:?}".sqlite}
: ${COMPOSE_FILE:="${LOCAL_RUN_DIR:?}"/compose.yaml}
: ${SERVER_ROOT:="${LOCAL_RUN_DIR:?}"/fs-root}
: ${ETC_PROSODY_DIR:="${SERVER_ROOT:?}"/etc/prosody}
: ${VAR_LIB_PROSODY_DIR:="${SERVER_ROOT:?}"/var/lib/prosody}

source "${SCRIPTS_ROOT:?}"/constants.sh
source "${SCRIPTS_ROOT:?}"/image-names.sh
