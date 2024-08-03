#!/bin/bash

image_name() {
	echo "${PROSE_DOCKER_ORG:?}/${1:?}:${DOCKER_TAG:?}"
}
archive_name() {
	echo $@ | sed -e 's#/#%2F#' -e 's/:/%3A/'
}

PROSE_DOCKER_ORG=proseim
DOCKER_TAG=latest
PROSE_POD_SERVER_IMAGE=$(image_name prose-pod-server)
PROSE_POD_SERVER_ARCHIVE=$(archive_name $PROSE_POD_SERVER_IMAGE)
PROSE_POD_API_IMAGE=$(image_name prose-pod-api)
PROSE_POD_API_ARCHIVE=$(archive_name $PROSE_POD_API_IMAGE)
