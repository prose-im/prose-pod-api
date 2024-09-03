: ${BASH_TOOLBOX:="${SCRIPTS_ROOT:?}"/bash-toolbox}
# NOTE: `die` logs an error then exits.
source "${BASH_TOOLBOX:?}"/die.sh
# NOTE: `edo` supports dry mode via `DRY_MODE`. When not in dry mode,
#   logs the command as a `TRACE` and executes it.
source "${BASH_TOOLBOX:?}"/edo.sh
# NOTE: `log.sh` provides utility for logging at different levels.
source "${BASH_TOOLBOX:?}"/log.sh
# NOTE: `test-env.sh` provides utility for checking environment variables.
source "${BASH_TOOLBOX:?}"/test-env.sh

# - Sets `TARGET_ARCH` to the `cargo` target to use when building.
# - Sets `DOCKER_TARGET_PLATFORM` to the target Docker platform.
# - Sets `DOCKER_ARCH_PREFIX` so we can prefix image names to avoid conflicts
#   with the ones used locally.
change-build-target() {
	case "${1:-local}" in
		# Not cross-compiling.
		--*|local)
			# NOTE: The Docker container runs on Alpine Linux, hence the target architecture.
			TARGET_ARCH=x86_64-unknown-linux-musl
			DOCKER_TARGET_PLATFORM="$(docker version --format '{{.Server.Os}}/{{.Server.Arch}}')"
			unset DOCKER_ARCH_PREFIX ;;

		# Raspberry Pi 2&3.
		armv7-unknown-linux-musleabihf|rpi2|rpi3)
			# Overwrite if we used an alias.
			TARGET_ARCH=armv7-unknown-linux-musleabihf
			DOCKER_TARGET_PLATFORM=linux/arm/v7
			DOCKER_ARCH_PREFIX=arm32v7 ;;

		# Raspberry Pi 4&5.
		armv8-unknown-linux-musleabihf|rpi4|rpi5)
			# Overwrite if we used an alias.
			TARGET_ARCH=armv8-unknown-linux-musleabihf
			DOCKER_TARGET_PLATFORM=linux/arm/v8
			DOCKER_ARCH_PREFIX=arm64v8 ;;

		# TIP: See all possible `TARGET_ARCH` values using `cargo target list`.
		# TIP: See all possible `DOCKER_ARCH_PREFIX` values at [docker-library/official-images](https://github.com/docker-library/official-images?tab=readme-ov-file#architectures-other-than-amd64).
		*) die "Unknown architecture: $(format_code $1). Update $(format_hyperlink 'scripts/util.sh' "file://${SCRIPTS_ROOT:?}/util.sh") if you want to add support for it." ;;
	esac

	# NOTE: `PROSE_POD_*_ARCHIVE` variables have already been generated so
	#   the image names won't be prefixed on the RPi (that's what we want).
	: ${PREFIXED_PROSE_POD_API_IMAGE:="${DOCKER_ARCH_PREFIX+"${DOCKER_ARCH_PREFIX%/}/"}${PROSE_POD_API_IMAGE:?}"}
	PROSE_POD_API_IMAGE="${PREFIXED_PROSE_POD_API_IMAGE}"
	: ${PREFIXED_PROSE_POD_SERVER_IMAGE:="${DOCKER_ARCH_PREFIX+"${DOCKER_ARCH_PREFIX%/}/"}${PROSE_POD_SERVER_IMAGE:?}"}
	PROSE_POD_SERVER_IMAGE="${PREFIXED_PROSE_POD_SERVER_IMAGE}"
}
