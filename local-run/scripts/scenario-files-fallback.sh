use-default() {
	local var_name=${1:?"Must pass a variable name."}
	eval "var=\${$var_name}"
	local path="${2:?"Must pass a file path."}"
	if [ -z "${var-}" ]; then
		eval "$var_name='${SCENARIO_DIR:?}/${path}'"
	fi
	if [ "${SCENARIO_NAME:?}" != default ] && ! [ -f "${var}" ]; then
		eval "$var_name='${SCENARIOS_DIR:?}/default/${path}'"
		warn "Using $(format_code "$1") from default scenario at $(format_url "$var")."
	fi
}

use-default ENV_FILE local-run.env
use-default SCENARIO_CONSTANTS_FILE constants.sh
use-default PROSE_CONFIG_FILE prose.toml
use-default COREDNS_COREFILE coredns/Corefile
use-default DNS_ZONE_FILE dns-zone.zone
