# dLive REAPER Link

**Current version: v0.2.0-beta.2 — macOS tour-test build.**

Creates REAPER markers from dLive MixRack scene recalls. This version receives MIDI through Allen & Heath MIDI Control and sends markers to REAPER over OSC.

## Current features

- Banked scene decoding for dLive scenes 1–500.
- Recording Only and Always marker modes.
- App-managed scene lists: edit, create, load, Save As, rename, delete, and `.drlscene` import/export.
- Automatic startup with the saved MIDI input, waiting and reconnecting when that input becomes available.
- Automatic recovery when the selected MIDI input disappears. Stop Listening suppresses automatic restart until Start Listening is pressed in that session.
- Refresh controls in Live and Setup that preserve the selected input and leave healthy listening connected.
- Compact Live, Scene Lists, Setup, and Diagnostics views, with saved window size and position.

## Setup

1. Connect Allen & Heath MIDI Control to the MixRack and configure it to expose the scene-recall MIDI input.
2. In this app’s Setup view, select that MIDI input and configure the REAPER OSC host and port.
3. Enable OSC in REAPER. For Recording Only mode, configure REAPER OSC feedback to `127.0.0.1:49102` on the computer running this app.
4. Create, import, or load the appropriate scene list, choose the marker mode, and start listening from Live.

Listening means the selected MIDI port is open. A virtual MIDI port can remain available without a live desk connection, so this status does not confirm that the MixRack is reachable.

## Version and validation

Find the app version in Diagnostics and include it in issue reports. See [CHANGELOG.md](CHANGELOG.md) for version history and [TOUR_TEST_NOTES.md](TOUR_TEST_NOTES.md) for the existing test-note format.

For v0.2.0-beta.2, mocked recovery checks, native Start/Stop/Refresh checks, light/dark layouts at 420×260 and 320×220, existing Rust tests, and the macOS app build passed. Physical MIDI unplug/replug and end-to-end desk-to-REAPER recovery still need hardware verification.

## Planned next milestone

The proposed v0.3.0-beta.1 milestone is direct dLive TCP connectivity and menu bar operation. Optional Surface control for Go/Next/Previous is also being planned. These features are not included in v0.2.0-beta.2.

## Development

```sh
npm install
npm run tauri dev
```
