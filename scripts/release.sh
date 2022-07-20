#!/bin/bash

set -e

PROJ_ROOT="$(git rev-parse --show-toplevel)"
TMPDIR="${PROJ_ROOT}/tmp/rel"

test -z "${VERSION}" && echo "\$VERSION is not specified." && exit 1

test -z "${NAME}" && NAME="xquo"
test -z "${BINNAME}" && BINNAME="xquo"
test -z "${TARGET}" && TARGET="x86_64-unknown-linux-musl"
# test -z "${FEATURES}" && FEATURES="jemalloc"

if test ! -d  "${TMPDIR}" ; then
    mkdir -p "${TMPDIR}" 
fi

TARGETDIR="${PROJ_ROOT}/target/${TARGET}"

if test ! -f  "${TMPDIR}/crate_licenses.txt" ; then
    cargo install --quiet cargo-license
    cargo license -d -a --avoid-build-deps --avoid-dev-deps > "${TMPDIR}/crate_licenses.txt"
fi


cargo build --quiet --release --target="${TARGET}" --features="${FEATURES}"
strip "${TARGETDIR}/release/${BINNAME}"

tar -zcf "${TMPDIR}/${NAME}-${VERSION}-${TARGET}.tar.gz" \
    -C "${TARGETDIR}/release" \
    "${BINNAME}"