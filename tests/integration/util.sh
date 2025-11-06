abort() {
	if [ -n "${1-}" ]; then
		error "${1:?}"
	fi

	echo
	if [ -n "${GITHUB_ACTIONS-}" ]; then
		task integration-test:logs
	else
		warn "┌───────────────────────────────────────────────────┐"
		warn "│ Run $(format_code task integration-test:logs) to see the logs. │"
		warn "└───────────────────────────────────────────────────┘"
	fi
	echo

	# Do not stop the API after a failure to allow investigation.
	# NOTE: In the CI, it will be stopped anyway so it’s perfect.
	exit 1
}

wait_until_api_running() {
	local start=$(date +%s) now elapsed timeout=3
	while ! edo log_as_trace_ xh GET -Iq "${INTEGRATION_TEST_HOST:?}"/health --timeout $timeout -p=HBhm; do
		now=$(date +%s)
		elapsed=$((now - start))
		if (( elapsed >= $timeout )); then
			abort "API still unreachable after ${timeout:?}s."
		fi
	done
}
