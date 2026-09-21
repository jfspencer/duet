# Duet icon

The approved icon has a teal microphone on the left and a copper microphone on the right.
The platinum cradle suggests a subtle smile. Its stem and base preserve the microphone stand shape.

## Files

- `duet-icon-source.png`: the original approved artwork from the built-in image tool.
- `duet-icon.png`: the 1024 × 1024 PNG export, with an opaque background that fills the square.
- `duet.icns`: the macOS application icon, with sizes from 16 to 1024 pixels.

Only the approved design remains in this directory.
The export process uses `sips` and `iconutil` for size and format conversion.

## Application use

Run `scripts/package-macos.sh` from the repository root to build `Duet.app`.
The script copies `duet.icns` into the bundle's resources.
The bundle's `CFBundleIconFile` entry selects this file for Finder and the Dock.

The graphite background extends to every edge. Do not add a transparent margin or round the source image corners.
macOS applies the final icon mask. This follows [Apple's app icon guide](https://developer.apple.com/design/human-interface-guidelines/app-icons/).

The PNG files provide the source for other platform packages and future exports.

## Final image edit

Tool: built-in `image_gen.imagegen`.

Prompt:

```text
Use case: precise-object-edit.
Edit target: the approved Duet app icon in the reference image.
Task: Correct ONLY its outer framing for a macOS Dock icon. The current image has a pre-rounded graphite tile inset inside a transparent outer margin. Remove that entire outer margin and the baked-in tile silhouette. The result must be an edge-to-edge FULLY OPAQUE SQUARE IMAGE, with the graphite material covering every pixel, including all four corners. There must be ZERO transparent pixels, ZERO outer border, ZERO inset tile, ZERO pre-rounded corners, and ZERO external drop shadow. macOS will supply the final rounded mask.
Preserve the exact existing microphone artwork: tall teal capsule on the left, shorter copper capsule on the right, platinum smile-shaped cradle, central stem, rounded foot, geometry, subtle smile, satin material, colors, lighting, and internal shadows.
Reframe the same composition as if cropping away the empty margin and uniformly enlarging the remaining design about 18%, with the full microphone stand approximately 70% of the canvas height. Keep it centered and preserve every relative proportion. Keep the graphite background softly lit from upper left and darker below. Extend that same graphite surface into the four square corners, with no enclosing rim.
Return only the finished square icon artwork. This is a framing correction, not a design revision. No new details, no text, no symbols, no transparency.
```
