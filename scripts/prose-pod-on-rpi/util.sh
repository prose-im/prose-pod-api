source "${SCRIPTS_ROOT:?}"/util.sh

# ===== CONNECTING TO THE REMOTE =====

RPI_USER="${RPI_USER-prose}"

rpi_sftp() {
	# NOTE: Keep hard tabs here, as `<<-` only removes leading tab characters and not leading spaces.
	edo sftp "${RPI_USER:?}@${RPI_IP:?}" <<-EOF
		$@
		bye
		EOF
	return $?
}

rpi_ssh() {
	# Add 'ssh>' before each line so we can filter the output and only get a trace of the commands ran.
	local commands=()
	while IFS='' read -r line; do
		line=$(echo "$line" | sed 's/^[[:space:]]*//')
	  commands+=("echo \"ssh> $line\"")
	  commands+=("$line")
	done <<< "$@"
	commands=$(printf "%s\n" "${commands[@]}")

	# NOTE: Every SSH session starts with a message on `&2` and a big paragraph… it's verbose.
	#   `2>&1 | sed -n '/ssh>/,$p'` removes all text until the first `ssh>` is found.
	# NOTE: Keep hard tabs here, as `<<-` only removes leading tab characters and not leading spaces.
	(edo ssh "${RPI_USER:?}@${RPI_IP:?}" '2>&1' '|' sed -n '/ssh>/,$p') <<-EOF
		set -e
		${commands}
		exit
		EOF
	return $?
}
