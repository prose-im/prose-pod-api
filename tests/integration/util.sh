abort() {
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
