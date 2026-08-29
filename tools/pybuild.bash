#!/usr/bin/env bash
# pyactivate.bash — activate the project Python virtual-env and build the project.
# Usage:  source tools/pyactivate.bash

THIS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export WorkSpace="${THIS_DIR}/.."
cd "${WorkSpace}"
export VIRTUAL_ENV="${WorkSpace}/.venv"
if [[ ! -d "${VIRTUAL_ENV}" ]]; then
    python -m venv "${VIRTUAL_ENV}"
fi

if [[ -d "${VIRTUAL_ENV}/Scripts" ]]; then
    VENV_BIN="${VIRTUAL_ENV}/Scripts"
else
    VENV_BIN="${VIRTUAL_ENV}/bin"
fi

# Protect PATH from having repeated entries
case ":${PATH}:" in
    *":${VENV_BIN}:"*) ;;
    *) export PATH="${VENV_BIN}:${PATH}" ;;
esac

unset PYTHONHOME
hash -r 2>/dev/null

if ! command -v maturin &> /dev/null; then
    python -m pip install maturin
fi

echo "Building Rust project..."
cargo build

echo "Building Python extension..."
python -m maturin develop -m src/pykosh/Cargo.toml
