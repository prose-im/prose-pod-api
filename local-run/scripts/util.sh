wait_until_api_running() {
	local host="${1:-"${API_HOST:?}"}"
	local start=$(date +%s) now elapsed timeout=3
	while ! edo log_as_trace_ xh GET -Iq "${host:?}"/health --timeout $timeout -p=HBhm; do
		now=$(date +%s)
		elapsed=$((now - start))
		if (( elapsed >= $timeout )); then
			abort "API still unreachable after ${timeout:?}s."
		fi
	done
}
