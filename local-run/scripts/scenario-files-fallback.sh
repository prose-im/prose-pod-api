use-default() {
	local -n var=${1:?"Must pass a variable name."}
	local path="${2:?"Must pass a file path."}"
	if [ -z "${var-}" ]; then
		var="${SCENARIO_DIR:?}/${path}"
	fi
	if [ "${SCENARIO_NAME:?}" != default ] && ! [ -f "${var}" ]; then
		var="${SCENARIOS_DIR:?}"/default/"${path}"
		warn "Using $(format_code "$1") from default scenario at $(format_url "$var")."
	fi
}

use-default ENV_FILE local-run.env
use-default SCENARIO_CONSTANTS_FILE constants.sh
use-default PROSE_CONFIG_FILE Prose.toml
use-default COREDNS_COREFILE coredns/Corefile
use-default DNS_ZONE_FILE dns-zone.zone
