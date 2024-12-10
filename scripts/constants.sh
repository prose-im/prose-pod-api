# ===== Define local heplers =====

image_name_() {
	local name="${1:?}"
	echo "${PROSE_DOCKER_ORG:?}/${name:?}"
}
image_name_full_() {
	local name="${1:?}"
	local tag="${2:-"${DEFAULT_DOCKER_TAG:?}"}"
	echo "${name:?}:${tag:?}"
}

# ===== Define constants =====
# NOTE: The `: ${VAR_NAME:=value}` syntax initializes a variable only if it's unset or null.
#   It avoids resetting a variable when sourcing this file after the variable was overriden.

: ${PROSE_DOCKER_ORG:=proseim}
: ${DEFAULT_DOCKER_TAG:=latest}
# Tag used to reference images built locally (e.g. uncommitted code).
: ${LOCAL_DOCKER_TAG:=local}

: ${PROSE_POD_SERVER_IMAGE_NAME:=$(image_name_ prose-pod-server)}
: ${PROSE_POD_SERVER_IMAGE_TAG:=${DEFAULT_DOCKER_TAG:?}}
: ${PROSE_POD_SERVER_IMAGE:=$(image_name_full_ "${PROSE_POD_SERVER_IMAGE_NAME:?}" "${PROSE_POD_SERVER_IMAGE_TAG}")}

: ${PROSE_POD_API_IMAGE_NAME:=$(image_name_ prose-pod-api)}
: ${PROSE_POD_API_IMAGE_TAG:=${DEFAULT_DOCKER_TAG:?}}
: ${PROSE_POD_API_IMAGE:=$(image_name_full_ "${PROSE_POD_API_IMAGE_NAME:?}" "${PROSE_POD_API_IMAGE_TAG}")}
