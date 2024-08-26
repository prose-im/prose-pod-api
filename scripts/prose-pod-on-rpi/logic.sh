source "$(dirname $0)"/constants.sh
source "$(dirname $0)"/util.sh

build_images() {
	info 'Building Prose Pod Server…'
	edo docker buildx build \
		--platform linux/arm/v7 \
		-t "${PROSE_POD_SERVER_IMAGE:?}" \
		${NO_PULL} \
		"${PROSE_POD_SERVER_DIR:?}"
	info 'Building Prose Pod API…'
	traced . "${SCRIPTS_ROOT:?}"/build-image "${TARGET_ARCH:?}" ${API_BUILD_OPTIONS[@]}
}

save_images() {
	info 'Creating Prose Pod temporary directory (to save Docker images)…'
	edo mkdir -p "${PROSE_TMPDIR:?}"
	edo rm -rf "${PROSE_TMPDIR:?}"/*

	info 'Saving Docker images…'
	edo docker save -o "${PROSE_TMPDIR:?}/${PROSE_POD_SERVER_ARCHIVE:?}.tar" "${PROSE_POD_SERVER_IMAGE:?}"
	edo docker save -o "${PROSE_TMPDIR:?}/${PROSE_POD_API_ARCHIVE:?}.tar" "${PROSE_POD_API_IMAGE:?}"

	info 'Saved Docker images:'
	# NOTE: We use a temporary `cd` here so `du` doesn't output full paths.
	(cd "${PROSE_TMPDIR:?}"; du -h * | sort -hr | _log_as_info)
}

send_images() {
	info 'Sending the Docker images to the Raspberry Pi…'

	rpi_sftp "# Copy the Docker images
		mkdir /var/tmp/prose-pod
	  put \"${PROSE_TMPDIR:?}/${PROSE_POD_SERVER_ARCHIVE:?}.tar\" /var/tmp/prose-pod/
		put \"${PROSE_TMPDIR:?}/${PROSE_POD_API_ARCHIVE:?}.tar\" /var/tmp/prose-pod/"

	rpi_ssh "# Load the Docker images
		docker load -i /var/tmp/prose-pod/\"${PROSE_POD_SERVER_ARCHIVE:?}\".tar
		docker load -i /var/tmp/prose-pod/\"${PROSE_POD_API_ARCHIVE:?}\".tar

		# Delete the archives
		rm /var/tmp/prose-pod/*.tar"
}

update_remote_prose_pod_system() {
	info 'Copying prose-pod-system on the Raspberry Pi…'
	REMOTE_PROSE_POD_SYSTEM_DIR='prose-pod-system'
	rpi_ssh "rm -r '${REMOTE_PROSE_POD_SYSTEM_DIR}' 2>/dev/null || :"
	rpi_sftp "put -R '${PROSE_POD_SYSTEM_DIR:?}' ${REMOTE_PROSE_POD_SYSTEM_DIR}" 2>&1 | grep -v '^Entering '
	traced "$(dirname $0)"/cleanup
}
