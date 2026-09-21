# Duet icon

The approved icon has a teal microphone on the left and a copper microphone on the right.
The platinum cradle suggests a subtle smile. Its stem and base preserve the microphone stand shape.

## Files

- `duet-icon-source.png`: the original approved artwork from the built-in image tool.
- `duet-icon.png`: the 1024 × 1024 PNG export, with transparent outer corners.
- `duet.icns`: the macOS application icon, with sizes from 16 to 1024 pixels.

Only the approved design remains in this directory.
The export process uses `sips` and `iconutil` for size and format conversion.

## Application use

Run `scripts/package-macos.sh` from the repository root to build `Duet.app`.
The script copies `duet.icns` into the bundle's resources.
The bundle's `CFBundleIconFile` entry selects this file for Finder and the Dock.

The PNG files provide the source for other platform packages and future exports.
