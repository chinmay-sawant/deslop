#!/usr/bin/env bash

set -euo pipefail

repository="${DESLOP_REPOSITORY:-chinmay-sawant/deslop}"
version="${DESLOP_VERSION:-}"
action_ref="${DESLOP_ACTION_REF:-}"
temp_dir="${RUNNER_TEMP:-/tmp}"
install_dir="${temp_dir}/deslop-bin"

case "${RUNNER_OS:-}:${RUNNER_ARCH:-}" in
  Linux:X64)
    asset_name="deslop-linux-x86_64.tar.gz"
    binary_name="deslop"
    ;;
  macOS:X64)
    asset_name="deslop-macos-x86_64.tar.gz"
    binary_name="deslop"
    ;;
  macOS:ARM64)
    asset_name="deslop-macos-arm64.tar.gz"
    binary_name="deslop"
    ;;
  *)
    echo "Unsupported runner: ${RUNNER_OS:-unknown}/${RUNNER_ARCH:-unknown}" >&2
    exit 1
    ;;
esac

if [[ -z "$version" ]]; then
  if [[ "$action_ref" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    version="$action_ref"
  else
    version="latest"
  fi
fi

if [[ "$version" == "latest" ]]; then
  download_url="https://github.com/${repository}/releases/latest/download/${asset_name}"
else
  download_url="https://github.com/${repository}/releases/download/${version}/${asset_name}"
fi

archive_path="${temp_dir}/${asset_name}"

mkdir -p "$install_dir"
curl --fail --location --silent --show-error "$download_url" --output "$archive_path"
tar -xzf "$archive_path" -C "$install_dir"
chmod +x "${install_dir}/${binary_name}"
echo "$install_dir" >> "$GITHUB_PATH"
