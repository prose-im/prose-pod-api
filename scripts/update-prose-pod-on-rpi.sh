#!/bin/bash

# Configure the script to exit when a command fails.
set -e

# ===== CONSTANTS =====

source "$(dirname $0)"/constants.sh

# Temporary directory used by this script
PROSE_TMPDIR="${TMPDIR%/}"/org.prose.pod.test

# ===== HELPER FUNCTIONS =====

source "$(dirname $0)"/util.sh

rpi_sftp() {
	# NOTE: Keep hard tabs here, as `<<-` only removes leading tab characters and not leading spaces.
	edo sftp prose@"${RPI_IP:?}" <<-EOF
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
	(edo ssh prose@"${RPI_IP:?}" 2>&1 | sed -n '/ssh>/,$p') <<-EOF
		set -e
		${commands}
		exit
		EOF
	return $?
}

cleanup() {
	debug "Cleaning up…"

	if (( $DELETE_LOCAL_ARCHIVES )); then
		edo rm -rf "${PROSE_TMPDIR:?}"
	fi
}

# ===== ARGUMENT PARSING =====

BUILD_IMAGES=1 SAVE_IMAGES=1 SEND_IMAGES=1 DELETE_LOCAL_ARCHIVES=1 UPDATE_SYSTEM=1 FORCE=0
API_BUILD_PARAMS=()

for arg in "$@"; do
	case $arg in
		--debug)
			info 'Will build in debug mode'
			API_BUILD_PARAMS+=('--debug')
			;;
		--no-pull)
			info 'Will not pull referenced Docker images'
			NO_PULL=' --pull=false'
			;;
		--no-build)
			info 'Will not build Docker images'
			BUILD_IMAGES=0
			;;
		--no-save)
			info 'Will not save Docker images'
			SAVE_IMAGES=0
			;;
		--no-send)
			info 'Will not send Docker images'
			SEND_IMAGES=0
			;;
		--no-system-update)
			info 'Will not update `prose-pod-system`'
			UPDATE_SYSTEM=0
			;;
		--keep-archives)
			info 'Will not delete local image archives'
			DELETE_LOCAL_ARCHIVES=0;
			;;
		--api-build-param=*)
			API_BUILD_PARAMS+=("${arg#'--api-build-param='}")
			;;
		# The script won't stop you if passed flags don't go together
		--force)
			info "Let's hope you know what you're doing"
			FORCE=1
			;;
		*)
			die "Unknown argument: ${arg}"
			;;
	esac
done

if (( !$FORCE )); then
	(( $SAVE_IMAGES && !$BUILD_IMAGES )) && die 'Cannot save images without building them.'
	(( $SEND_IMAGES && !$SAVE_IMAGES )) && die 'Cannot send images without saving them.'
	(( !$BUILD_IMAGES && !$SEND_IMAGES && !$SAVE_IMAGES )) && die 'Seriously?'
fi

# ===== ENVIRONMENT CHECK =====

test_env_var() {
	[[ -z "${!1}" ]] && die "Please set \`$1\` to $2."
	return 0
}

test_env_var RPI_IP "the Raspberry Pi's IPv4 address"
test_env_var PATH_TO_PROSE_POD_SYSTEM 'the path to the `prose-pod-system` repository'
test_env_var PATH_TO_PROSE_POD_SERVER 'the path to the `prose-pod-server` repository'
test_env_var PATH_TO_PROSE_POD_API 'the path to the `prose-pod-api` repository'

# ===== MAIN LOGIC =====

# Register the cleanup function to be called on exit
trap cleanup EXIT

if (( $BUILD_IMAGES )); then
	debug 'Building Prose Pod Server…'
	edo docker buildx build \
		--platform linux/arm/v7 \
		-t "${PROSE_POD_SERVER_IMAGE:?}" \
		${NO_PULL} \
		"${PATH_TO_PROSE_POD_SERVER:?}"
	debug 'Building Prose Pod API…'
	"$(dirname $0)"/build-image.sh armv7-unknown-linux-musleabihf ${API_BUILD_PARAMS[@]}
fi

if (( $SAVE_IMAGES )); then
	debug 'Creating Prose Pod temporary directory (to save Docker images)…'
	edo mkdir -p "${PROSE_TMPDIR:?}"
	edo rm -rf "${PROSE_TMPDIR:?}"/*

	debug 'Saving Docker images…'
	edo docker save -o "${PROSE_TMPDIR:?}/${PROSE_POD_SERVER_ARCHIVE:?}.tar" "${PROSE_POD_SERVER_IMAGE:?}"
	edo docker save -o "${PROSE_TMPDIR:?}/${PROSE_POD_API_ARCHIVE:?}.tar" "${PROSE_POD_API_IMAGE:?}"

	info 'Saved Docker images:'
	# NOTE: We use a temporary `cd` here so `du` doesn't output full paths
	(cd "${PROSE_TMPDIR:?}"; du -h * | sort -hr)
fi

if (( $SEND_IMAGES )); then
	debug 'Sending the Docker images to the Raspberry Pi…'

	rpi_sftp "# Copy the Docker images
		mkdir /var/tmp/prose-pod
	  put \"${PROSE_TMPDIR:?}/${PROSE_POD_SERVER_ARCHIVE:?}.tar\" /var/tmp/prose-pod/
		put \"${PROSE_TMPDIR:?}/${PROSE_POD_API_ARCHIVE:?}.tar\" /var/tmp/prose-pod/"

	rpi_ssh "# Load the Docker images
		docker load -i /var/tmp/prose-pod/\"${PROSE_POD_SERVER_ARCHIVE:?}\".tar
		docker load -i /var/tmp/prose-pod/\"${PROSE_POD_API_ARCHIVE:?}\".tar

		# Delete the archives
		rm /var/tmp/prose-pod/*.tar"
fi

if (( $UPDATE_SYSTEM )); then
	debug 'Copying prose-pod-system on the Raspberry Pi…'
	rpi_ssh 'rm -r prose-pod-system 2>/dev/null || :'
	rpi_sftp "put -R \"${PATH_TO_PROSE_POD_SYSTEM:?}\" prose-pod-system" 2>&1 | grep -v '^Entering '
	rpi_ssh '# Clean things up
		rm prose-pod-system/server/pod/etc/prosody/prosody.cfg.lua 2>/dev/null || :;
		rm -r prose-pod-system/server/pod/var/lib/prosody/*%2e* 2>/dev/null || :;'
fi
