# ===== Source higher-level constants =====

source "${SCRIPTS_ROOT:?}"/constants.sh

# ===== Define constants =====
# NOTE: The `: ${VAR_NAME:=value}` syntax initializes a variable only if it's unset or null
#   and the `: ${VAR_NAME=value}` syntax initializes a variable only if it's unset.
#   It avoids resetting a variable when sourcing this file after the variable was overriden.

# Temporary directory used by this script.
: ${PROSE_TMPDIR:="${TMPDIR%/}"/org.prose.pod.test}

: ${REMOTE_PROSE_POD_SYSTEM_DIR="prose-pod-system"}
: ${REMOTE_SERVER_ROOT="${REMOTE_PROSE_POD_SYSTEM_DIR:?}"/server/pod}
