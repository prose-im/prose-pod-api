BASH_TOOLBOX="${SCRIPTS_ROOT:?}"/bash-toolbox
# NOTE: `die` logs an error then exits.
source "${BASH_TOOLBOX:?}"/die.sh
# NOTE: `edo` supports dry mode via `DRY_MODE`. When not in dry mode,
#   logs the command as a `TRACE` and executes it.
source "${BASH_TOOLBOX:?}"/edo.sh
# NOTE: `log.sh` provides utility for logging at different levels.
source "${BASH_TOOLBOX:?}"/log.sh
# NOTE: `test-env.sh` provides utility for checking environment variables.
source "${BASH_TOOLBOX:?}"/test-env.sh
