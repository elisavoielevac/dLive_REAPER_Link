# Changelog

## v0.3.0-beta.2 — 2026-11-02

- Add an Ethernet interface picker for Direct MixRack TCP, showing adapter names and IP addresses.
- Remember the selected adapter and bind every connection/reconnect to it. Never use Wi-Fi or silently switch adapters; wait when the selected adapter is unavailable.
- Require an explicit Ethernet selection when updating older TCP settings. Preserve MIDI input mode and existing playback-recall protections.
- Update the illustrated guide, offline README, and wired-network setup instructions, including control-network bridging and overlapping-subnet troubleshooting.
- User testing confirmed physical MixRack scene recalls, direct and patched Waves V3 connections, and automatic recovery after cable/adapter loss. Cable-only recovery was about 2.5 seconds with Wi-Fi off; overlapping Wi-Fi/Ethernet subnets can delay recovery.

## v0.3.0-beta.1 - first packaged release - 2026-10-24

Release preparation update: add the MIT license and a reproducible signed build command that remaps local home-folder paths out of the distributed executable. Re-sign, notarize, and verify the updated installer; application behavior is unchanged.

- Package a universal macOS beta with Developer ID signing and Apple notarization, an installer DMG, offline README, updated illustrated user guide, and third-party notices.
- Move Live into a compact 320px-wide menu bar dropdown. Grow vertically to fit warnings without scrolling; omit dropdown navigation because Preferences is available in the right-click menu.
- Open Scene Lists, Setup, and Diagnostics in a separate resizable settings window with saved size and position. Launch in the menu bar; closing settings keeps the bridge running.
- Group settings sections and scene-list actions, remove the repeated list heading, align fields/actions, and label diagnostic values.
- Include optional playback recall and the REAPER OSC setup guide described below.

## Development work included in the first release — 2026-10-14

- Add a four-page illustrated PDF user guide and real macOS app/REAPER screenshots in the README.
- Add an expandable REAPER OSC setup guide with exact field names, port directions, another-computer setup, initial refresh, and explicit test-marker labeling.
- Add playback/current-marker feedback and Diagnostics preview using exact marker-name-to-scene-name matching, preserving original scene slots.
- Add MixRack bank/program-change sending through the active TCP connection, with bounded writes, connection-specific sends, and no command queue or retry.
- Add an off-by-default Playback recall toggle. Recall only when the mapped scene differs from the last sent scene; retain that memory across Stop/Play, Pause/Resume, and same-song loops.
- Exclude recording, suppress desk-to-REAPER marker creation while recall is enabled, and protect queued input and delayed echoes after disabling.
- Disarm on connection/source/settings/list changes or lost playback feedback, requiring explicit re-enable.
- Accept OSC feedback only from the configured REAPER host and clear old playback state when the target changes.

## v0.3.0-beta.1 development baseline — 2026-10-13

- Add direct plain MixRack TCP reception alongside selectable MIDI input fallback.
- Decode split/coalesced MIDI streams and running status, with configurable desk MIDI channel.
- Move listening intent, serialized connection ownership, and recovery into Rust.
- Add TCP keepalive and discard queued recalls from lost or replaced connections.
- Keep REAPER marker waits separate from connection reading; preserve scene lists and marker modes.
- Add menu bar status and controls; the open window appears in the Dock and Command-Tab. Closing Preferences hides it and switches to menu-bar-only operation; reopening Preferences restores Dock/Command-Tab access. Quit exits.
- Migrate saved MIDI input selections and require Save source to apply connection settings.

## v0.2.0-beta.2 — 2026-10-13

- Automatically reconnect the selected/saved MIDI input, including when it is missing at startup.
- Show Waiting while an armed listener awaits its input; keep an unavailable selection visible in Setup.
- Let Stop Listening cancel automatic recovery until an explicit Start in that session.
- Add a compact Live Refresh control using the same recovery logic as Setup Refresh, without interrupting healthy listening.
- Serialize connection attempts and reuse MIDI event subscriptions across reconnects.
- Refine the compact macOS utility layout, support 420×260 default and 320×220 minimum windows, and preserve window size and position.
- Keep the version in Diagnostics and update the README to describe the working app and its MIDI Control dependency.

## v0.2.0-beta.1

- dLive MIDI scene recall detection with banked decoding above scene 128.
- REAPER OSC marker creation and shared OSC feedback listener.
- Recording Only marker mode.
- App-managed scene lists with editor, create, load, Save As, rename, delete, and `.drlscene` import/export.
- Compact Live, Scene Lists, Setup, and Diagnostics UI with small-window usability improvements.
