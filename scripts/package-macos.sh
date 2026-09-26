#!/usr/bin/env bash
set -euo pipefail

case "${1:-}" in
  "") duet_profile=dev ;;
  --release) duet_profile=release ;;
  -h|--help)
    printf 'Usage: scripts/package-macos.sh [--release]\n'
    exit 0
    ;;
  *)
    printf 'Usage: scripts/package-macos.sh [--release]\n' >&2
    exit 2
    ;;
esac

if [[ $# -gt 1 ]]; then
  printf 'Usage: scripts/package-macos.sh [--release]\n' >&2
  exit 2
fi

if [[ "$(uname -s)" != Darwin ]]; then
  printf 'package-macos: this command requires macOS.\n' >&2
  exit 1
fi

duet_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$duet_root"

duet_build_executable() {
  local message
  while IFS= read -r message; do
    case "$message" in
      *'"executable":null'*) ;;
      *'"reason":"compiler-artifact"'*)
        if [[ "$(plutil -extract target.name raw -expect string -o - - <<< "$message")" == duet ]]; then
          plutil -extract executable raw -expect string -o - - <<< "$message"
        fi
        ;;
    esac
  done
}

duet_executable="$(cargo build --locked --package duet --bin duet --profile "$duet_profile" \
  --message-format=json-render-diagnostics | duet_build_executable)"
if [[ -z "$duet_executable" || ! -x "$duet_executable" ]]; then
  printf 'package-macos: Cargo did not produce the Duet executable.\n' >&2
  exit 1
fi

duet_package_id="$(cargo pkgid --locked --package duet)"
duet_version="${duet_package_id##*#}"
duet_version="${duet_version##*@}"
duet_bundle="$(dirname "$duet_executable")/Duet.app"
duet_contents="$duet_bundle/Contents"

mkdir -p "$duet_contents/MacOS" "$duet_contents/Resources"
cp "$duet_executable" "$duet_contents/MacOS/duet"
cp crates/bc_app/duet/assets/icons/duet.icns "$duet_contents/Resources/duet.icns"
cp crates/bc_app/duet/packaging/macos/Info.plist "$duet_contents/Info.plist"
plutil -replace CFBundleShortVersionString -string "$duet_version" "$duet_contents/Info.plist"
plutil -replace CFBundleVersion -string "$duet_version" "$duet_contents/Info.plist"
plutil -lint "$duet_contents/Info.plist" >&2
printf '%s\n' "$duet_bundle"
