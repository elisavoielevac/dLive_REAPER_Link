# Changelog

## v0.2.0-beta.2 — 2026-10-13

- Automatically reconnect the selected/saved MIDI input, including when it is missing at startup.
- Show Waiting while an armed listener awaits its input; keep an unavailable selection visible in Setup.
- Let Stop Listening cancel automatic recovery until an explicit Start in that session.
- Add a compact Live Refresh control using the same recovery logic as Setup Refresh, without interrupting healthy listening.
- Serialize connection attempts and reuse MIDI event subscriptions across reconnects.
- Refine the compact macOS utility layout, support 420×260 default and 320×220 minimum windows, and preserve window size and position.
- Keep the version in Diagnostics and update the README to describe the working app and its MIDI Control dependency.

Validation: mocked recovery scenarios, native Start/Stop/Refresh, default/minimum light/dark layouts, existing Rust tests, and a macOS app build passed. Physical MIDI unplug/replug and end-to-end desk-to-REAPER recovery remain unverified.

## v0.2.0-beta.1

- dLive MIDI scene recall detection with banked decoding above scene 128.
- REAPER OSC marker creation and shared OSC feedback listener.
- Recording Only marker mode.
- App-managed scene lists with editor, create, load, Save As, rename, delete, and `.drlscene` import/export.
- Compact Live, Scene Lists, Setup, and Diagnostics UI with small-window usability improvements.
