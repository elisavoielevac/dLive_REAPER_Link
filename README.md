# dLive REAPER Link

**First packaged release: v0.3.0-beta.1 — macOS beta.**

Creates REAPER markers from dLive MixRack scene recalls, with optional REAPER playback-to-MixRack scene recall for virtual soundcheck. Choose direct MixRack MIDI-over-TCP or an existing MIDI input exposed by Allen & Heath MIDI Control. Playback recall requires Direct MixRack TCP.

## Download and installation

Download the universal DMG from [Releases](https://github.com/elisavoielevac/dLive_REAPER_Link/releases). Open it, drag **dLive REAPER Link** to **Applications**, eject the image, and launch the Applications copy. The app starts in the menu bar; right-click its icon for **Preferences…** and **Quit**.

The release includes the illustrated PDF manual, an offline installation README, third-party notices, and SHA-256 checksums. The app and DMG are Developer ID-signed and Apple-notarized. The universal app contains Apple Silicon and Intel code. Intel execution and installation on another Mac are unverified. The bundle declares macOS 10.13 as its minimum; older macOS versions have not been tested.

This is a beta: Director and simulated-desk results do not establish physical MixRack/Surface or cable-loss recovery validation. Test on your own setup before a show.

## User guide

Download the [illustrated PDF user guide](docs/dLive-REAPER-Link-User-Guide.pdf). The app figures show the current release frontend rendered with simulated example data. The REAPER configuration screenshot was captured in REAPER v7.80. An editable [Markdown manual](docs/USER_MANUAL.md) is also included in the repository.

![Compact Live dropdown with example data](docs/screenshots/live.png)

## Current features

- MixRack scenes 1–500, with configurable desk MIDI channel (1–12).
- Direct plain TCP connection with stream framing, MIDI running status, reconnect, and TCP keepalive.
- MIDI input fallback; only one selected source creates markers.
- Recording Only and Always marker modes.
- Off-by-default playback recall, using exact REAPER marker names and the selected scene list’s original scene-slot numbers. Preview and send status appear in Diagnostics.
- App-managed scene lists: edit, create, load, Save As, rename, delete, and `.drlscene` import/export.
- Rust-owned startup and recovery, independent of the Preferences window. Saved sources start automatically; missing sources wait and retry about every two seconds after a failed attempt.
- Stop Listening cancels recovery until Start Listening in that session. Reconnect does not override Stop. Refresh displays status without restarting healthy listening.
- Click the menu bar icon for a compact 320px-wide Live dropdown that grows vertically to fit warnings without scrolling. Click outside it or press Escape to dismiss it. Right-click the icon and choose Preferences… to open Scene Lists, Setup, and Diagnostics in a separate larger settings window with saved size and position.
- The settings window appears in the Dock and Command-Tab; its red close button hides it while the bridge stays active. Right-click the menu bar icon for status, Start/Stop Listening, Reconnect, Preferences…, and Quit. The app launches in the menu bar; Quit exits.

## Setup

1. In Setup, choose **Direct MixRack TCP** and enter the MixRack IP, plain TCP port (normally `51325`), and the desk's base MIDI channel. This implementation targets dLive firmware V2.0 and later. Enable outgoing MIDI on the desk. TLS ports and Surface cue recall are not supported in this beta.
2. Alternatively, connect Allen & Heath MIDI Control to the MixRack and select its MIDI input. Existing saved MIDI selections migrate automatically. Choose **Save source** to apply source settings.
3. Configure the REAPER OSC host and port, and enable OSC in REAPER. For Recording Only mode, configure REAPER OSC feedback to `127.0.0.1:49102` on the computer running this app.
4. Create, import, or load the appropriate scene list, choose the marker mode, and start listening from Live or the menu bar.

Green means the selected TCP socket or MIDI input is open. Amber means waiting/connecting, gray means stopped, and red means an error. A virtual MIDI port can remain present without a live desk. An open TCP connection does not confirm that outgoing scene MIDI is enabled or that REAPER received a marker. Check Diagnostics and use the separate REAPER test.

TCP keepalive helps detect silent network loss without treating an idle desk as disconnected. Detection time depends on the operating system; physical cable-loss timing still needs hardware testing. Pending scene recalls from a lost or replaced connection are discarded. With Playback recall Off, the desk connection remains receive-only. Turning Playback recall On enables scene commands to the MixRack.

## REAPER OSC configuration

The app has an expandable **Setup → REAPER OSC → Set up OSC in REAPER** guide.

![Real REAPER OSC settings dialog](docs/screenshots/reaper-osc.png)

The device name “Marker Test” in this screenshot is an example; any device name is acceptable. Local IP is REAPER’s displayed address.

OSC works in both directions: the app sends marker commands to REAPER’s listen port, while REAPER sends transport/marker feedback to this app on port 49102.

For both apps on the same Mac, open **REAPER Preferences → Control/OSC/web**, then Add an OSC entry or Edit the entry dedicated to this app:

| REAPER field | Value |
| --- | --- |
| Control surface mode | OSC (Open Sound Control) |
| Device name | dLive REAPER Link (any name) |
| Pattern config | Default |
| Mode | Configure device IP+local port |
| Device IP | 127.0.0.1 (the app’s feedback destination) |
| Device port | 49102 (feedback into the app) |
| Local listen port | 49101, or the port saved in this app (commands into REAPER) |

Confirm with OK. In this app, set **REAPER IP** to 127.0.0.1 and **Port** to the same Local listen port, then Save. Leave other OSC settings unchanged. Default.ReaperOSC already supplies the required action, transport, time, and last-marker mappings; no custom pattern file is required.

If REAPER runs on another computer, this app’s REAPER IP must be that computer’s IP. REAPER’s Device IP must be the IP of the computer running this app. Device port remains 49102; Local listen port still matches the app’s Port.

Run **Control surface: Refresh all surfaces** in REAPER’s Actions list and allow a couple of seconds for Diagnostics to show transport and current-marker feedback. Playback recall needs a marker at or before the cursor and a matching scene name. The app must also have a listening Direct MixRack TCP connection before recall can be enabled.

![Setup view with example data](docs/screenshots/setup.png)

The app screenshot uses illustrative addresses. Replace the MixRack IP with your desk address; the same-computer REAPER address remains 127.0.0.1.

**Test marker** creates a marker named `REAPER CONNECTION TEST` in the active REAPER session. Use a disposable test session for that check.

## Playback recall

Playback recall lets REAPER control MixRack scene changes during playback, making it useful for virtual soundcheck. As playback reaches each song marker, the app recalls the scene with the same name from the selected scene list.

For example, a marker named `Song A` recalls the scene named `Song A`. The scene list determines its desk scene number; REAPER marker numbers do not affect the mapping. Names must match exactly, and blank scene slots keep their original numbers. Unmatched markers cause no recall; duplicate scene names prevent enabling the feature.

Playback recall requires **Direct MixRack TCP**, a listening desk connection, and REAPER OSC transport and marker feedback. Turn it on using **Playback recall** in the menu bar dropdown. It starts **Off** whenever the app launches. Enabling it during playback can immediately recall the current song's scene.

While it is on:

- Scene changes follow playback, including seeking or looping into another song.
- Stopping and restarting, pausing and resuming, or looping within the same song does not repeatedly recall the same scene.
- Recording does not trigger scene recalls, and incoming desk recalls do not create REAPER markers.
- Lost connections, stale feedback during playback, or changes to connection settings or the scene list turn recall Off. Turn it on again after the connection and settings are ready.
- Closing settings leaves playback recall running.

Diagnostics shows the mapped scene and send status. **Command sent** does not confirm that the desk recalled the scene. Playback recall targets MixRack scenes; it does not move the Surface cue list.

## Version and support

Find the app version in Diagnostics and include it when reporting an issue, along with your setup and what happened. See [CHANGELOG.md](CHANGELOG.md) for version history and [TOUR_TEST_NOTES.md](TOUR_TEST_NOTES.md) for an issue-report template.

## Later work

Optional independent Surface control for OSC-triggered Go/Next/Previous is deferred. Surface cue recall IDs must never be used as MixRack scene marker numbers. TLS and virtual MIDI output routing are also deferred.

## License

Licensed under the [MIT License](LICENSE), copyright 2026 Éli Savoie-Levac. Third-party dependencies retain their own licenses; see [third-party notices](THIRD_PARTY_NOTICES.txt).

## Development

```sh
npm install
npm run tauri dev
```

The illustrated manual source is `docs/USER_MANUAL.md`. Rebuild its PDF with `python3 docs/build_user_guide.py` using Python with ReportLab installed.

For signed universal release builds, set `APPLE_SIGNING_IDENTITY` to your Developer ID Application identity and run `python3 scripts/build_release.py`. It remaps local build paths so your home-folder name is not embedded in Rust diagnostic strings.
