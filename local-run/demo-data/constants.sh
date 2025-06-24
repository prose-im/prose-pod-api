#!/usr/bin/env bash

# Configure the script to exit when a command fails.
set -e

: ${REPOSITORY_ROOT:="${PROSE_POD_API_DIR:?}"}
DEMO_DATA_DIR="${REPOSITORY_ROOT:?}/local-run/demo-data"
AVATARS_DIR="${DEMO_DATA_DIR:?}/avatars"

DOMAIN='example.org'
WORKSPACE_NAME='Prose (demo)'
PASSWORD='demo'

MEMBER_ROLES=()
MEMBER_SEXTYPES=()
MEMBER_FIRSTNAMES=()
MEMBER_LASTNAMES=()
MEMBER_NICKNAMES=()
MEMBER_USERNAMES=()
MEMBER_JIDS=()
MEMBER_EMAILS=()
MEMBER_AVATARS=()
MEMBER_PASSWORDS=()
MEMBER_TOKENS=()

jid() {
	echo "${1:?}@${DOMAIN:?}"
}

add-member() {
	MEMBER_ROLES+=("${1:?}")
	MEMBER_SEXTYPES+=("${2:?}")
	MEMBER_FIRSTNAMES+=("${3:?}")
	MEMBER_LASTNAMES+=("${4:?}")
	MEMBER_NICKNAMES+=("${5-}")
	local username="${6:?}"
	MEMBER_USERNAMES+=("${username:?}")
	local jid="$(jid "${username:?}")"
	MEMBER_JIDS+=("${jid:?}")
	MEMBER_EMAILS+=("${7:?}")
	local avatar_file=
	if [ -n "${8-}" ]; then
		local avatar="$(cat "${AVATARS_DIR:?}/${8:?}" | base64)"
		MEMBER_AVATARS+=("${avatar:?}")
	else
		MEMBER_AVATARS+=('')
	fi
	# NOTE: Default password is the user’s JID (thanks to `debug_only.insecure_password_on_auto_accept_invitation = true`).
	MEMBER_PASSWORDS+=("${9:-"${jid:?}"}")
	MEMBER_TOKENS+=('not-logged-in')
}

# NOTE: https://faker-playground.vercel.app

# 1
add-member ADMIN female Pauline Collins 'Pauline C.' 'pauline.collins' 'pauline.collins@example.org' 'openPeeps-a.png' "${PASSWORD:?}"
# 2
add-member MEMBER male Todd Schultz 'Todd S.' 'todd.schultz' 'todd.schultz@example.net' 'openPeeps-b.png'
# 3
add-member MEMBER female Sheri Nienow 'Sheri N.' 'sheri.nienow' 'sheri.nienow@example.net' 'openPeeps-c.png'
# 4
add-member ADMIN male Evan Turner 'Evan T.' 'evan.turner' 'evan.turner@example.com' 'openPeeps-d.png'
# 5
add-member MEMBER male Charlie Schmitt 'Charlie S.' 'charlie.schmitt' 'charlie.schmitt@example.org' 'openPeeps-e.png'
# 6
add-member MEMBER female Meghan Donnelly 'Meghan D.' 'meghan.donnelly' 'meghan.donnelly@example.net' 'openPeeps-f.png'
# 7
add-member MEMBER female Ella White 'Ella W.' 'ella.white' 'ella.white@example.com' 'openPeeps-g.png'
# 8
add-member MEMBER male Taylor Cartwright 'Taylor C.' 'taylor.cartwright' 'taylor.cartwright@example.org' 'openPeeps-h.png'
# 9
add-member MEMBER female Dianna Hermann '' 'dianna.hermann' 'dianna.hermann@example.net' 'openPeeps-i.png'
# 10
add-member MEMBER male Jake Lang 'Jake L.' 'jake.lang' 'jake.lang@example.com' ''
# 11
add-member MEMBER male Pablo Bartoletti 'Pablo B.' 'pablo.bartoletti' 'pablo.bartoletti@example.com' 'openPeeps-k.png'
# 12
add-member ADMIN female Barbara Mante 'Barbara M.' 'barbara.mante' 'barbara.mante@example.com' 'openPeeps-l.png'
# 13
add-member MEMBER male Drew Howell 'Drew H.' 'drew.howell' 'drew.howell@example.com' 'openPeeps-m.png'

# Member 9 has no nickname
# Member 10 has no avatar
