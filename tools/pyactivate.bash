#!/usr/bin/env bash
# pyactivate.bash — activate the project Python virtual-env.
# Usage:  source tools/pyactivate.bash

THIS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export WorkSpace="${THIS_DIR}/.."
cd "${WorkSpace}"
if [[ ! -d ".venv" ]]; then
    python -m venv .venv
fi
export VIRTUAL_ENV="${WorkSpace}/.venv"
export PATH="${VIRTUAL_ENV}/Scripts:${PATH}"

unset PYTHONHOME

hash -r 2>/dev/null
