# dLive REAPER Link

**Current version: v0.3.0-beta.1 — macOS hardware-test build.**

Creates REAPER markers from dLive MixRack scene recalls. Choose direct MixRack MIDI-over-TCP or an existing MIDI input exposed by Allen & Heath MIDI Control.

## Current features

- MixRack scenes 1–500, with configurable desk MIDI channel (1–12).
- Direct plain TCP connection with stream framing, MIDI running status, reconnect, and TCP keepalive.
- MIDI input fallback; only one selected source creates markers.
- Recording Only and Always marker modes.
- App-managed scene lists: edit, create, load, Save As, rename, delete, and `.drlscene` import/export.
- Rust-owned startup and recovery, independent of the Preferences window. Saved sources start automatically; missing sources wait and retry about every two seconds after a failed attempt.
- Stop Listening cancels recovery until Start Listening in that session. Reconnect does not override Stop. Refresh displays status without restarting healthy listening.
- Menu bar status, Start/Stop Listening, Reconnect, Preferences…, and Quit. The open window appears in the Dock and Command-Tab. Clicking the red close button hides it and keeps the bridge alive in the menu bar; Preferences… restores the window, Dock, and Command-Tab. Quit exits.
- Compact Live, Scene Lists, Setup, and Diagnostics views, with saved window size and position.

## Setup

1. In Setup, choose **Direct MixRack TCP** and enter the MixRack IP, plain TCP port (normally `51325`), and the desk's base MIDI channel. This implementation targets dLive firmware V2.0 and later. Enable outgoing MIDI on the desk. TLS ports and Surface cue recall are not supported in this beta.
2. Alternatively, connect Allen & Heath MIDI Control to the MixRack and select its MIDI input. Existing saved MIDI selections migrate automatically. Choose **Save source** to apply source settings.
3. Configure the REAPER OSC host and port, and enable OSC in REAPER. For Recording Only mode, configure REAPER OSC feedback to `127.0.0.1:49102` on the computer running this app.
4. Create, import, or load the appropriate scene list, choose the marker mode, and start listening from Live or the menu bar.

Green means the selected TCP socket or MIDI input is open. Amber means waiting/connecting, gray means stopped, and red means an error. A virtual MIDI port can remain present without a live desk. An open TCP connection does not confirm that outgoing scene MIDI is enabled or that REAPER received a marker. Check Diagnostics and use the separate REAPER test.

TCP keepalive helps detect silent network loss without treating an idle desk as disconnected. Detection time depends on the operating system; physical cable-loss timing still needs hardware testing. Pending scene recalls from a lost or replaced connection are discarded. No scene recall commands are sent to the desk in this version.

## Version and validation

Find the app version in Diagnostics and include it in issue reports. See [CHANGELOG.md](CHANGELOG.md) for version history and [TOUR_TEST_NOTES.md](TOUR_TEST_NOTES.md) for the existing test-note format.

Automated tests cover TCP framing, scene boundaries and channels, missing-at-start recovery, disconnect/reconnect, Stop/rearm, endpoint changes, settings migration, and discarding stale queued recalls. Mocked browser checks cover source selection, listening controls, enumeration failure, event subscriptions, and default/minimum light/dark layouts.

Native localhost scene reception, Start/Stop/Refresh, version display, window close survival, keyboard Quit, and the macOS app build passed. User testing confirmed scene reception with dLive Director and reported the initial window/Command-Tab behavior working. The automated UI tool could not inspect the native tray dropdown. Physical MixRack reception, MIDI unplug/replug, silent cable-loss detection, and end-to-end desk-to-REAPER recovery remain unverified.

## Later work

Optional independent Surface control for OSC-triggered Go/Next/Previous is deferred. Surface cue recall IDs must never be used as MixRack scene marker numbers. TLS, direct scene recall sending, and virtual MIDI routing are also deferred.

## Development

```sh
npm install
npm run tauri dev
```
