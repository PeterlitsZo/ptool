#!/usr/bin/env bash

set -euo pipefail

BASE_URL="https://peterlits.net/ptool/release/latest"
INSTALL_DIR="${HOME}/.local/bin"
tmpdir=""

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'Error: required command not found: %s\n' "$1" >&2
    exit 1
  fi
}

detect_asset() {
  local os
  local arch

  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64|amd64)
          printf '%s\n' 'linux-amd64'
          ;;
        aarch64|arm64)
          printf '%s\n' 'linux-arm64'
          ;;
        x86|i386|i486|i586|i686)
          printf '%s\n' 'linux-x86'
          ;;
        arm|armv6l|armv7l)
          printf '%s\n' 'linux-arm'
          ;;
        riscv64|riscv64gc)
          printf '%s\n' 'linux-riscv64'
          ;;
        *)
          printf 'Error: unsupported Linux architecture: %s\n' "$arch" >&2
          exit 1
          ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        x86_64|amd64)
          printf '%s\n' 'macos-amd64'
          ;;
        aarch64|arm64)
          printf '%s\n' 'macos-arm64'
          ;;
        *)
          printf 'Error: unsupported macOS architecture: %s\n' "$arch" >&2
          exit 1
          ;;
      esac
      ;;
    *)
      printf 'Error: unsupported operating system: %s\n' "$os" >&2
      exit 1
      ;;
  esac
}

cleanup() {
  if [ -n "${tmpdir:-}" ] && [ -d "$tmpdir" ]; then
    rm -rf -- "$tmpdir"
  fi
}

main() {
  local asset archive_name archive_url archive_path extracted_path dest_path

  need_cmd curl
  need_cmd tar
  need_cmd mktemp

  asset="$(detect_asset)"
  archive_name="ptool-${asset}.tar.gz"
  archive_url="${BASE_URL}/${archive_name}"

  tmpdir="$(mktemp -d)"
  trap cleanup EXIT

  archive_path="${tmpdir}/${archive_name}"
  extracted_path="${tmpdir}/ptool"
  dest_path="${INSTALL_DIR}/ptool"

  printf 'Downloading %s\n' "$archive_url"
  curl -fsSL "$archive_url" -o "$archive_path"

  mkdir -p "$INSTALL_DIR"

  printf 'Installing to %s\n' "$dest_path"
  tar -xzf "$archive_path" -C "$tmpdir"

  if [ ! -f "$extracted_path" ]; then
    printf 'Error: archive did not contain a ptool binary.\n' >&2
    exit 1
  fi

  cp "$extracted_path" "$dest_path"
  chmod 0755 "$dest_path"

  printf 'Installed ptool to %s\n' "$dest_path"
  printf 'If `ptool` is not found, add %s to your PATH.\n' "$INSTALL_DIR"
}

main "$@"
