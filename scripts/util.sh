#!/bin/bash

trace() {
	echo "[TRACE] $@"
	return $?
}
debug() {
	echo "[DEBUG] $@"
	return $?
}
info() {
	echo " [INFO] $@"
	return $?
}
error() {
	echo "[ERROR] $@"
	return $?
}

edo() {
	trace $@
	$@
	return $?
}
die() {
	error $@
	exit 1
}
