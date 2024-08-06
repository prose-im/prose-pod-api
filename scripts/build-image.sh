#!/bin/bash

# Configure the script to exit when a command fails.
set -e

# ===== CONSTANTS =====

source "$(dirname $0)"/constants.sh

# ===== HELPER FUNCTIONS =====

source "$(dirname $0)"/util.sh

usage() {
	cat <<-EOF
	Usage:
	  $(basename $0) TARGET_ARCH
	    Build the '${PROSE_POD_API_IMAGE:?}' Docker image.
	  $(basename $0) [--]help
	    Print this help.

	Test:
	EOF
}
help() {
	usage
	exit 0
}

# ===== ARGUMENT PARSING =====

[[ $# > 0 ]] || die $(usage)

TARGET_ARCH=${1:?}
case "${TARGET_ARCH}" in
	x86_64-unknown-linux-musl)
		DOCKER_TARGET_PLATFORM=linux/amd64 ;;
	armv7-unknown-linux-musleabihf)
		DOCKER_TARGET_PLATFORM=linux/arm/v7 ;;
	--help) help ;;
	*) die "Unknown architecture: '${TARGET_ARCH}'" ;;
esac

# Skip already interpreted argument
shift 1

CARGO_PROFILE=release RUST_BUILD_MODE=release
SKIP_RUST_BUILD=0

for arg in "$@"; do
	case $arg in
		--debug)
			info 'Will build in debug mode'
			CARGO_PROFILE=dev
			RUST_BUILD_MODE=debug
			;;
		--skip-rust-build)
			info 'Will not build the Rust project'
			SKIP_RUST_BUILD=1
			;;
		--no-pull)
			info 'Will not pull referenced Docker images'
			NO_PULL=' --pull=false'
			;;
		--help) help ;;
		*) die "Unknown argument: ${arg}\n$(usage)" ;;
	esac
done

# ===== MAIN LOGIC =====

info "Using $(rustc --version), $(rustup --version 2>/dev/null) and $(cargo --version)"

if (( !$SKIP_RUST_BUILD )); then
	edo cross build --target "${TARGET_ARCH}" --profile "${CARGO_PROFILE}"
fi

edo docker buildx build \
	--platform "${DOCKER_TARGET_PLATFORM:?}" \
	-t "${PROSE_POD_API_IMAGE:?}" \
	--build-arg RUST_OUT_DIR="./target/${TARGET_ARCH}/${RUST_BUILD_MODE:?}" \
	${NO_PULL} \
	"${PATH_TO_PROSE_POD_API:?}"
