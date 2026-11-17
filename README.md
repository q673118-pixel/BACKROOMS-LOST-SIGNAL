# BACKROOMS: LOST SIGNAL

A first-person atmospheric horror prototype built with Rust and Bevy, targeting Android.

## Current prototype scope

- Main menu with Play, Test (Debug World), and Settings.
- Separate Debug World with primitive Level 0 geometry, material samples, door geometry, collision volumes, and sparse localized glitch patches.
- Mobile touch input: left-side virtual movement area and right-side camera look.
- Settings categories: Controls, VHS, Graphics, Audio, and Interface.
- Settings are serialized to a JSON file in the app's writable home directory.
- Android APK build through GitHub Actions.

## Important prototype limitations

This is an early technical prototype, not a finished game. The Play screen is currently a placeholder. Materials are procedural color materials rather than final seamless texture assets. VHS values are stored and exposed in the settings UI; a dedicated post-processing shader and the individual visual implementations (noise, chromatic aberration, distortion, flicker) are still pending. Audio controls are scaffolding until audio assets and playback systems are added. The FPS selector is currently a stored preference, not an enforced frame limiter.

## Build

The Android workflow builds an ARM64 APK and uploads it as the artifact `backrooms-lost-signal-debug-apk`.

Open the repository's **Actions** tab, select **Android APK**, and inspect the latest run. When it succeeds, download the APK from the workflow run's Artifacts section.

## Technology

- Rust
- Bevy 0.20
- Android GameActivity
- Gradle / Android SDK / NDK
