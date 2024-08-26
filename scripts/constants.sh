# ===== Define local heplers =====

image_name_() {
	echo "${PROSE_DOCKER_ORG:?}/${1:?}:${DOCKER_TAG:?}"
}
archive_name_() {
	echo $@ | sed -e 's#/#%2F#' -e 's/:/%3A/'
}

# ===== Define constants =====
# NOTE: The `: ${VAR_NAME:=value}` syntax initializes a variable only if it's unset or null.
#   It avoids resetting a variable when sourcing this file after the variable was overriden.

: ${PROSE_DOCKER_ORG:=proseim}
: ${DOCKER_TAG:=latest}
: ${PROSE_POD_SERVER_IMAGE:=$(image_name_ prose-pod-server)}
: ${PROSE_POD_SERVER_ARCHIVE:=$(archive_name_ $PROSE_POD_SERVER_IMAGE)}
: ${PROSE_POD_API_IMAGE:=$(image_name_ prose-pod-api)}
: ${PROSE_POD_API_ARCHIVE:=$(archive_name_ $PROSE_POD_API_IMAGE)}
