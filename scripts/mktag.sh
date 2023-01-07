#!/bin/bash
set -euo pipefail

if [[ $# -lt 2 ]]; then
    echo "usage: $0 image_name (stable|stable-git|dev-git)..."
    exit 2
fi

cd "$(dirname "$0")/.."
seg_git="$(git rev-parse --short HEAD)"
seg_yymmdd="$(date --utc +%Y%m%d)"
seg_rfc3339="$(date --utc --rfc-3339=seconds)"

img="$1"; shift

declare -a tags
for x in $*; do
    case $x in
    "stable")
        tags+=(
            "${img}:${seg_git}"
        ) ;;
    "stable-git")
        tags+=(
            "${img}:${seg_yymmdd}.git${seg_git}"
        ) ;;
    "dev-git")
        tags+=(
            "${img}:dev.${seg_yymmdd}.git${seg_git}"
        ) ;;
    *)
        echo "invalid argument: $x" >&2
        exit 1
        ;;
    esac
done

printf "%s\n" "${tags[@]}"

if [[ ${GITHUB_ACTIONS-} == "true" ]]; then
    echo "::set-output name=tags::$(IFS=","; echo "${tags[*]}")"
    echo "::set-output name=git::${seg_git}"
    echo "::set-output name=yymmdd::${seg_yymmdd}"
    echo "::set-output name=rfc3339::${seg_rfc3339}"
fi