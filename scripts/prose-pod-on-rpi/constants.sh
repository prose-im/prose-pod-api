source "${SCRIPTS_ROOT:?}"/constants.sh

# Temporary directory used by this script.
PROSE_TMPDIR="${TMPDIR%/}"/org.prose.pod.test

# TODO: Allow choosing `TARGET_ARCH` and `DOCKER_ARCH_PREFIX` via command line arguments
#   to better support other Raspberry Pi versions.

# `cargo` target to use when building.
# TIP: See all possible values using `cargo target list`.
TARGET_ARCH="${TARGET_ARCH-"armv7-unknown-linux-musleabihf"}"
# Prefix image names so it doesn't conflict with the ones used locally.
# NOTE: Value comes from <https://github.com/docker-library/official-images?tab=readme-ov-file#architectures-other-than-amd64>.
DOCKER_ARCH_PREFIX="${DOCKER_ARCH_PREFIX-"arm32v7"}"
# NOTE: `PROSE_POD_*_ARCHIVE` variables have already been generated so
#   the image names won't be prefixed on the RPi (that's what we want).
PROSE_POD_API_IMAGE="${DOCKER_ARCH_PREFIX}/${PROSE_POD_API_IMAGE:?}"
PROSE_POD_SERVER_IMAGE="${DOCKER_ARCH_PREFIX}/${PROSE_POD_SERVER_IMAGE:?}"

REMOTE_PROSE_POD_SYSTEM_DIR=${REMOTE_PROSE_POD_SYSTEM_DIR-"prose-pod-system"};
REMOTE_SERVER_ROOT=${REMOTE_SERVER_ROOT-"${REMOTE_PROSE_POD_SYSTEM_DIR:?}"/server/pod};
