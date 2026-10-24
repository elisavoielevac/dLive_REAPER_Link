dLive REAPER Link - v0.3.0-beta.1
First packaged macOS beta - 24 October 2026

INSTALL
Open the DMG, drag dLive REAPER Link to Applications, eject the image,
and launch the Applications copy. Click the menu bar icon for Live.
Right-click it for Preferences and Quit. Closing settings keeps the bridge running.

QUICK SETUP
1. Preferences > Setup: choose Direct MixRack TCP, enter the desk IP,
   plain TCP port (normally 51325), and base MIDI channel (1-12).
   Enable outgoing MIDI on the desk, then Save source.
   Alternatively choose a MIDI input from Allen & Heath MIDI Control.
2. Configure REAPER OSC. On the same Mac: app host 127.0.0.1, port 49101;
   REAPER Device IP 127.0.0.1, Device port 49102, Local listen port 49101,
   Pattern config Default, Mode Configure device IP+local port.
3. Run Control surface: Refresh all surfaces manually in REAPER.
4. Load the correct scene list. Line number equals original desk scene slot;
   preserve empty lines. Choose Recording Only or Always and start listening.

PLAYBACK RECALL
Optional and Off at every launch. Requires Direct MixRack TCP, fresh
REAPER feedback, and exact marker-name-to-scene-name matches.
Enabling during playback can immediately recall the current mapped scene.
Recording does not recall; incoming desk recalls do not create markers
while playback recall is On. Connection/settings/list changes disarm it.
A successful command write does not confirm a physical desk recall.
The feature targets MixRack scenes, not the Surface cue list.

READ THE INCLUDED PDF
The illustrated User Manual covers ports, separate computers, scene lists,
playback recall, troubleshooting, and test limits. App figures use simulated
example values. Test marker adds REAPER CONNECTION TEST to the active
REAPER project: use a disposable project.

COMPATIBILITY AND BETA LIMITS
Universal binary: Apple Silicon and Intel code. Intel execution and
installation on another Mac are unverified. Bundle minimum macOS: 10.13;
older macOS versions have not been tested. Director/software testing passed;
physical MixRack/Surface, MIDI unplug/replug, silent cable-loss timing,
and end-to-end hardware recovery remain unverified.
App and DMG are Developer ID-signed and Apple-notarized.

SUPPORT AND UPDATES
https://github.com/elisavoielevac/dLive_REAPER_Link
Report version, macOS/Mac model, source/IP/port/channel, marker mode,
playback recall state, exact names/slots, transport state, and reproduction steps.

Third-party licenses and source links are included in THIRD_PARTY_NOTICES.txt.

LICENSE
Copyright (c) 2026 Éli Savoie-Levac. Distributed under the MIT License;
see the included LICENSE file. Dependencies retain their respective licenses.
