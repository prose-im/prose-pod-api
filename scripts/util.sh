: ${BASH_TOOLBOX:="${SCRIPTS_ROOT:?}/bash-toolbox"}

# Initialize submodules if someone didn't do it first.
[ -f "${BASH_TOOLBOX:?}"/README.md ] || git -C "${SCRIPTS_ROOT:?}" submodule update --init

# NOTE: `die` logs an error then exits.
source "${BASH_TOOLBOX:?}"/die.sh
# NOTE: `edo` supports dry mode via `DRY_MODE`. When not in dry mode,
#   logs the command as a `TRACE` and executes it.
source "${BASH_TOOLBOX:?}"/edo.sh
# NOTE: `log.sh` provides utility for logging at different levels.
source "${BASH_TOOLBOX:?}"/log.sh
# NOTE: `test-env.sh` provides utility for checking environment variables.
source "${BASH_TOOLBOX:?}"/test-env.sh

# - Sets `DOCKER_TARGET_PLATFORM` to the target Docker platform.
change-build-target() {
	case "${1:-local}" in
		# Not cross-compiling.
		--*|local)
			unset DOCKER_TARGET_PLATFORM ;;

		# Raspberry Pi 2&3.
		rpi2|rpi3)
			# Overwrite if we used an alias.
			DOCKER_TARGET_PLATFORM=linux/arm/v7 ;;

		# Raspberry Pi 4&5.
		rpi4|rpi5)
			# Overwrite if we used an alias.
			DOCKER_TARGET_PLATFORM=linux/arm/v8 ;;

		# Other (we'll let Docker complain if the value is incorrect).
		*)
			DOCKER_TARGET_PLATFORM="$1" ;;
	esac
}

# Redirect users to the docs if env vars are not set.
# NOTE: We still use `${X:?}` in the rest of the script, this is just to improve the dev UX.
# COPYRIGHT: <https://stackoverflow.com/a/65396324/10967642>.
test-env-vars() {
	local doc_file="$1"
	shift 1

	local var_names=("$@") var_name
	for var_name in "${var_names[@]}"; do
		[ -z "${!var_name}" ] && echo "${var_name} isn't set. Check $(format_url "${doc_file}") for more information." && var_unset=true
	done
	[ -n "$var_unset" ] && exit 1
	return 0
}

traced-export() {
	local var_name
	for var_name in "$@"; do
		export ${var_name}
		trace "${var_name}=${!var_name}"
	done
}
