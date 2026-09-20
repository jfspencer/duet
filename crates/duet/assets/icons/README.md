# Duet icon

The icon combines two vocal channels into one microphone shape. Platinum and teal distinguish the two channels on a graphite tile.

## Files

- `duet-icon-source.png`: the original artwork from the built-in image tool.
- `duet-icon.png`: the 1024 × 1024 PNG export, with transparent outer corners.
- `duet.icns`: the macOS icon file, with sizes from 16 to 1024 pixels.

The app has no bundle configuration. These files are ready for a future app bundle.

## Source

Tool: built-in `image_gen.imagegen`. The export process uses `sips` and `iconutil` for size and format conversion.

Prompt:

```text
Use case: logo-brand
Asset type: final desktop application icon for Duet, a professional music composition, vocal recording, mixing, and mastering application.
Primary request: Create one distinctive, polished app icon that appeals to vocal artists and studio engineers. Clear iconography and instant recognition at small sizes matter most.
Subject: one bold custom studio microphone symbol with a paired-voice idea built into it. The microphone capsule consists of exactly two close, parallel, upright rounded pillars of equal width, with the right pillar slightly shorter. One smooth, thick U-shaped pickup cradle surrounds their lower ends and joins a short central stem and a simple broad rounded foot. The two capsule pillars suggest two voices and the paired channels of an audio console. It must read first as a single elegant microphone, with an original compact silhouette.
Style: precise geometric app icon with restrained dimensional craft. Broad simple shapes, optically balanced, clean rounded edges. The symbol has a soft satin-metal finish with subtle bevels. No fine texture or intricate grille.
Composition: one icon only, front-facing, centered on a square 1024 by 1024 canvas. The dark graphite rounded-square tile is inset about 7% from the canvas edges. The symbol occupies about 60% of the tile height, with generous balanced breathing room. True transparent alpha outside the rounded-square tile. No background scene or presentation board.
Color and materials: graphite tile, softly shaded from charcoal at the top to almost black below. Warm platinum for the left pillar and cradle, a restrained cool teal for the right pillar. Strong contrast. Subtle controlled upper-left studio light, crisp edges, very shallow depth.
Constraints: a professional creative instrument, suitable for a desktop Dock and application launcher. The mark must remain legible at 32 pixels. No text, no wordmark, no tiny details, no buttons, no extra music notes, no headphones, no separate waveform decoration, no neon glow, no lens flare, no heavy chrome, no glass, no perspective tilt, no watermark, no mockup, no collage. Render the usable icon asset itself.
```

