#!/usr/bin/env bash
# SPEC-016 — installs the prebuilt skillport release binary for the runner's
# OS/arch, verifying its sha256, falling back (signal only, never hard-fail)
# when no prebuilt binary is available. Used by action.yml's "Install
# skillport (prebuilt)" step; NOT responsible for the `cargo install`
# fallback itself (that's a separate, gated action.yml step).
#
# Asset naming/layout MUST match .github/workflows/release.yml (SPEC-014)
# exactly: skillport-<ver>-<triple>.<ext>, staged dir
# skillport-<ver>-<triple>/<binary> inside the archive.
set -euo pipefail

REPO="jysf/skillport"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.skillport/bin}"

PRINT_PLAN=false
VERSION_INPUT="${VERSION:-latest}"

# --- arg parsing -------------------------------------------------------
while [ $# -gt 0 ]; do
  case "$1" in
    --print-plan)
      PRINT_PLAN=true
      shift
      ;;
    --version)
      VERSION_INPUT="$2"
      shift 2
      ;;
    *)
      echo "install-release.sh: unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

RUNNER_OS="${RUNNER_OS:-}"
RUNNER_ARCH="${RUNNER_ARCH:-}"

# --- output helpers ------------------------------------------------------
# Emits installed=<bool> to $GITHUB_OUTPUT if set (real GitHub Actions runs);
# always echoes to stdout too so a local/test invocation can see it.
emit_installed() {
  local value="$1"
  echo "installed=${value}"
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    echo "installed=${value}" >> "$GITHUB_OUTPUT"
  fi
}

# Signals a recoverable miss: log + installed=false + exit 0. Never used for
# genuinely unexpected errors (e.g. a corrupt archive after a good download).
fallback() {
  local reason="$1"
  echo "install-release.sh: falling back to source install: ${reason}" >&2
  emit_installed "false"
  exit 0
}

# --- platform map (SPEC-016 table, must match SPEC-014 matrix) -----------
# Sets TRIPLE, EXT, BINARY as globals; returns 1 (does not exit) for an
# unsupported pair so callers can decide to print-plan or fall back.
resolve_platform() {
  case "${RUNNER_OS}/${RUNNER_ARCH}" in
    Linux/X64)
      TRIPLE="x86_64-unknown-linux-gnu"; EXT="tar.gz"; BINARY="skillport" ;;
    Linux/ARM64)
      TRIPLE="aarch64-unknown-linux-musl"; EXT="tar.gz"; BINARY="skillport" ;;
    macOS/X64)
      TRIPLE="x86_64-apple-darwin"; EXT="tar.gz"; BINARY="skillport" ;;
    macOS/ARM64)
      TRIPLE="aarch64-apple-darwin"; EXT="tar.gz"; BINARY="skillport" ;;
    Windows/X64)
      TRIPLE="x86_64-pc-windows-msvc"; EXT="zip"; BINARY="skillport.exe" ;;
    *)
      TRIPLE=""; EXT=""; BINARY=""
      return 1
      ;;
  esac
  return 0
}

SUPPORTED=true
if ! resolve_platform; then
  SUPPORTED=false
fi

# --- version resolution ---------------------------------------------------
# print-plan mode must never touch the network: fabricate a tag from the
# input instead of resolving "latest" via the GitHub API.
resolve_tag() {
  if [ "$VERSION_INPUT" = "latest" ]; then
    if [ "$PRINT_PLAN" = "true" ]; then
      echo "latest"
      return 0
    fi
    local tag
    tag="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null \
      | grep -m1 '"tag_name"' | sed -E 's/.*"tag_name":[[:space:]]*"([^"]+)".*/\1/')" || true
    if [ -z "$tag" ]; then
      return 1
    fi
    echo "$tag"
    return 0
  fi
  echo "v${VERSION_INPUT}"
}

TAG="$(resolve_tag || true)"
if [ -z "$TAG" ]; then
  if [ "$PRINT_PLAN" = "true" ]; then
    TAG="latest"
  else
    fallback "could not resolve release tag for version '${VERSION_INPUT}' (no release yet?)"
  fi
fi
VER="${TAG#v}"

ASSET=""
URL=""
if [ "$SUPPORTED" = "true" ]; then
  ASSET="skillport-${VER}-${TRIPLE}.${EXT}"
  URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET}"
fi

# --- --print-plan: no network, just report the resolved plan -------------
if [ "$PRINT_PLAN" = "true" ]; then
  echo "os=${RUNNER_OS}"
  echo "arch=${RUNNER_ARCH}"
  echo "triple=${TRIPLE}"
  echo "ext=${EXT}"
  echo "version=${VER}"
  echo "asset=${ASSET}"
  echo "url=${URL}"
  echo "supported=${SUPPORTED}"
  exit 0
fi

# --- real run --------------------------------------------------------------
if [ "$SUPPORTED" != "true" ]; then
  fallback "unsupported platform: RUNNER_OS=${RUNNER_OS} RUNNER_ARCH=${RUNNER_ARCH}"
fi

WORKDIR="$(mktemp -d)"
cleanup() { rm -rf "$WORKDIR"; }
trap cleanup EXIT

ARCHIVE_PATH="${WORKDIR}/${ASSET}"
CHECKSUM_PATH="${ARCHIVE_PATH}.sha256"

if ! curl -fsSL -o "$ARCHIVE_PATH" "$URL"; then
  fallback "asset not found (404 or network error): ${URL}"
fi
if ! curl -fsSL -o "$CHECKSUM_PATH" "${URL}.sha256"; then
  fallback "checksum file not found: ${URL}.sha256"
fi

# Verify checksum from within the download dir so the recorded filename
# (no path prefix) matches what sha256sum/shasum expects.
if ! (
  cd "$WORKDIR"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum -c "$(basename "$CHECKSUM_PATH")"
  else
    shasum -a 256 -c "$(basename "$CHECKSUM_PATH")"
  fi
); then
  fallback "checksum verification failed for ${ASSET}"
fi

# Extract — a failure here is unexpected (a good download + good checksum
# should always extract), so this is allowed to hard-fail per the spec.
case "$EXT" in
  tar.gz)
    tar xzf "$ARCHIVE_PATH" -C "$WORKDIR"
    ;;
  zip)
    unzip -q "$ARCHIVE_PATH" -d "$WORKDIR"
    ;;
  *)
    fallback "unknown archive extension: ${EXT}"
    ;;
esac

STAGE_DIR="${WORKDIR}/skillport-${VER}-${TRIPLE}"
BINARY_PATH="${STAGE_DIR}/${BINARY}"
if [ ! -f "$BINARY_PATH" ]; then
  echo "install-release.sh: extracted archive missing expected binary at ${BINARY_PATH}" >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR"
mv "$BINARY_PATH" "$INSTALL_DIR/$BINARY"
chmod +x "$INSTALL_DIR/$BINARY" 2>/dev/null || true

if [ -n "${GITHUB_PATH:-}" ]; then
  echo "$INSTALL_DIR" >> "$GITHUB_PATH"
fi

echo "install-release.sh: installed ${BINARY} ${VER} (${TRIPLE}) to ${INSTALL_DIR}" >&2
emit_installed "true"
