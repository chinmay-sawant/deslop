#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
extension_dir="${script_dir}/../vscode-finding-opener"

if ! command -v code >/dev/null 2>&1; then
  echo "The VS Code CLI ('code') was not found in PATH." >&2
  echo "Open VS Code and run 'Shell Command: Install \"code\" command in PATH' first." >&2
  exit 1
fi

if ! command -v npm >/dev/null 2>&1; then
  echo "npm was not found in PATH." >&2
  exit 1
fi

cd "$extension_dir"
npm ci
npm run compile
npx --yes vsce@2.15.0 package --out deslop-finding-opener.vsix
code --install-extension deslop-finding-opener.vsix --force
