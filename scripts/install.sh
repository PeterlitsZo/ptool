#!/usr/bin/env bash

set -euo pipefail

RELEASE_BASE_URL="https://peterlits.net/ptool/release"
DEFAULT_INSTALL_DIR="${HOME}/.local/bin"
tmpdir=""
DEBUG="${DEBUG:-0}"

debug() {
  if [ "$DEBUG" = "1" ]; then
    printf '[DEBUG] %s\n' "$*" >&2
  fi
}

debug_file_size() {
  local path size

  if [ "$DEBUG" != "1" ] || [ ! -f "$1" ]; then
    return 0
  fi

  path="$1"
  size="$(wc -c < "$path")"
  size="${size//[[:space:]]/}"
  debug "file size: ${path} (${size} bytes)"
}

debug_directory_state() {
  local label path

  if [ "$DEBUG" != "1" ]; then
    return 0
  fi

  label="$1"
  path="$2"

  debug "${label}: ${path}"

  if [ ! -e "$path" ]; then
    debug "${label} does not exist"
    return 0
  fi

  ls -ld "$path" >&2 || true

  if [ -d "$path" ]; then
    debug "contents of ${path}:"
    ls -la "$path" >&2 || true
  fi

  if command -v df >/dev/null 2>&1; then
    debug "filesystem usage for ${path}:"
    df -h "$path" >&2 || true
    df -i "$path" >&2 || true
  fi
}

debug_archive_state() {
  local archive_path

  if [ "$DEBUG" != "1" ]; then
    return 0
  fi

  archive_path="$1"

  debug_file_size "$archive_path"
  debug "archive entries in ${archive_path}:"
  tar -tzf "$archive_path" >&2 || debug "failed to list archive entries"
}

debug_failure_context() {
  local step archive_path extracted_path dest_path

  if [ "$DEBUG" != "1" ]; then
    return 0
  fi

  step="$1"
  archive_path="$2"
  extracted_path="$3"
  dest_path="$4"

  debug "failure while ${step}"
  debug "working directory: $(pwd)"
  debug "TMPDIR environment: ${TMPDIR:-<unset>}"
  debug_directory_state "temporary directory" "$tmpdir"
  debug_directory_state "install directory" "$INSTALL_DIR"
  debug_archive_state "$archive_path"

  if [ -e "$extracted_path" ]; then
    debug_directory_state "extracted file" "$extracted_path"
    debug_file_size "$extracted_path"
  fi

  if [ -e "$dest_path" ]; then
    debug_directory_state "destination file" "$dest_path"
    debug_file_size "$dest_path"
  fi
}

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'Error: required command not found: %s\n' "$1" >&2
    exit 1
  fi

  debug "found command: $1 -> $(command -v "$1")"
}

usage() {
  cat >&2 <<'EOF'
Usage: install.sh [--bin-dir=<path>] [<tag>]

Install the latest stable release by default, or install a specific release tag
such as v0.2.0 or v0.2.0-alpha.1.

Options:
  --bin-dir=<path>   Install the ptool binary into <path> instead of ~/.local/bin.

Environment:
  DEBUG=1            Print extra debug information during installation.
EOF
}

parse_args() {
  INSTALL_DIR="${DEFAULT_INSTALL_DIR}"
  RELEASE_TAG=""

  while [ "$#" -gt 0 ]; do
    case "$1" in
      --bin-dir)
        if [ "$#" -lt 2 ]; then
          printf 'Error: --bin-dir requires a path.\n' >&2
          usage
          exit 1
        fi
        INSTALL_DIR="$2"
        shift 2
        ;;
      --bin-dir=*)
        INSTALL_DIR="${1#--bin-dir=}"
        shift
        ;;
      --help|-h)
        usage
        exit 0
        ;;
      v*)
        if [ -n "$RELEASE_TAG" ]; then
          printf 'Error: expected at most one release tag argument.\n' >&2
          usage
          exit 1
        fi
        if [[ "$1" == */* ]]; then
          printf 'Error: release tag must not contain `/`: %s\n' "$1" >&2
          usage
          exit 1
        fi
        RELEASE_TAG="$1"
        shift
        ;;
      *)
        printf 'Error: unrecognized argument: %s\n' "$1" >&2
        usage
        exit 1
        ;;
    esac
  done

  if [ -z "$INSTALL_DIR" ]; then
    printf 'Error: --bin-dir must not be empty.\n' >&2
    usage
    exit 1
  fi

  debug "parsed arguments: install_dir=${INSTALL_DIR}, release_tag=${RELEASE_TAG:-latest}"
}

detect_asset() {
  local os
  local arch

  os="$(uname -s)"
  arch="$(uname -m)"

  debug "detected platform: os=${os}, arch=${arch}"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64|amd64)
          debug "selected asset: linux-amd64"
          printf '%s\n' 'linux-amd64'
          ;;
        aarch64|arm64)
          debug "selected asset: linux-arm64"
          printf '%s\n' 'linux-arm64'
          ;;
        x86|i386|i486|i586|i686)
          debug "selected asset: linux-x86"
          printf '%s\n' 'linux-x86'
          ;;
        arm|armv6l|armv7l)
          debug "selected asset: linux-arm"
          printf '%s\n' 'linux-arm'
          ;;
        riscv64|riscv64gc)
          debug "selected asset: linux-riscv64"
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
          debug "selected asset: macos-amd64"
          printf '%s\n' 'macos-amd64'
          ;;
        aarch64|arm64)
          debug "selected asset: macos-arm64"
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
    debug "cleaning up temporary directory: $tmpdir"
    rm -rf -- "$tmpdir"
  fi
}

main() {
  local asset archive_name archive_url archive_path extracted_path dest_path
  local release_path
  local -a curl_args

  need_cmd curl
  need_cmd tar
  need_cmd mktemp

  parse_args "$@"
  asset="$(detect_asset)"
  debug "working directory: $(pwd)"
  debug "TMPDIR environment: ${TMPDIR:-<unset>}"

  if [ -n "$RELEASE_TAG" ]; then
    release_path="${RELEASE_BASE_URL}/${RELEASE_TAG}"
    archive_name="ptool-${RELEASE_TAG}-${asset}.tar.gz"
  else
    release_path="${RELEASE_BASE_URL}/latest"
    archive_name="ptool-${asset}.tar.gz"
  fi

  archive_url="${release_path}/${archive_name}"
  debug "resolved release path: ${release_path}"
  debug "resolved archive url: ${archive_url}"

  tmpdir="$(mktemp -d)"
  trap cleanup EXIT
  debug "created temporary directory: ${tmpdir}"

  archive_path="${tmpdir}/${archive_name}"
  extracted_path="${tmpdir}/ptool"
  dest_path="${INSTALL_DIR}/ptool"
  debug "archive will be stored at: ${archive_path}"
  debug "binary will be extracted from: ${extracted_path}"
  debug "binary will be installed to: ${dest_path}"

  printf 'Downloading %s\n' "$archive_url"
  curl_args=(-fsSL "$archive_url" -o "$archive_path")
  if [ "$DEBUG" = "1" ]; then
    curl_args=(-fSL --verbose "$archive_url" -o "$archive_path")
  fi
  curl "${curl_args[@]}"
  debug_archive_state "$archive_path"

  mkdir -p "$INSTALL_DIR"
  debug "ensured install directory exists: ${INSTALL_DIR}"
  debug_directory_state "temporary directory" "$tmpdir"
  debug_directory_state "install directory" "$INSTALL_DIR"

  printf 'Installing to %s\n' "$dest_path"
  if ! tar -xzf "$archive_path" -C "$tmpdir"; then
    debug_failure_context "extracting the release archive" "$archive_path" "$extracted_path" "$dest_path"
    exit 1
  fi
  debug "archive extracted into: ${tmpdir}"

  if [ ! -f "$extracted_path" ]; then
    printf 'Error: archive did not contain a ptool binary.\n' >&2
    exit 1
  fi

  debug_file_size "$extracted_path"

  if ! cp "$extracted_path" "$dest_path"; then
    debug_failure_context "copying the extracted binary" "$archive_path" "$extracted_path" "$dest_path"
    exit 1
  fi
  chmod 0755 "$dest_path"
  debug "installed executable permissions on: ${dest_path}"
  debug_file_size "$dest_path"

  printf 'Installed ptool to %s\n' "$dest_path"
  printf 'If `ptool` is not found, add %s to your PATH.\n' "$INSTALL_DIR"
}

main "$@"
