const { invoke } = window.__TAURI__.core;

window.addEventListener("DOMContentLoaded", () => {
  const reaperOscHost = document.querySelector("#reaper-osc-host");
  const reaperOscPort = document.querySelector("#reaper-osc-port");
  const saveReaperOscSettingsButton = document.querySelector(
    "#save-reaper-osc-settings",
  );
  const testReaperConnectionButton = document.querySelector(
    "#test-reaper-connection",
  );
  const reaperOscSettingsResult = document.querySelector(
    "#reaper-osc-settings-result",
  );
  const midiButton = document.querySelector("#list-midi-inputs");
  const midiResult = document.querySelector("#midi-inputs-result");
  const sceneFilePath = document.querySelector("#scene-file-path");
  const revealSceneFileButton = document.querySelector("#reveal-scene-file");
  const reloadSceneNamesButton = document.querySelector("#reload-scene-names");
  const sceneFileRevealResult = document.querySelector(
    "#scene-file-reveal-result",
  );
  const sceneFileReloadResult = document.querySelector(
    "#scene-file-reload-result",
  );
  const midiInputSelect = document.querySelector("#midi-input-select");
  const startMidiLogButton = document.querySelector("#start-midi-log");
  const stopMidiLogButton = document.querySelector("#stop-midi-log");
  const midiLogResult = document.querySelector("#midi-log-result");
  const midiSceneResult = document.querySelector("#midi-scene-result");
  const midiSceneNameResult = document.querySelector("#midi-scene-name-result");
  const midiLog = document.querySelector("#midi-log");
  let dLiveLinkRunning = false;
  let unlistenMidiMessages;

  async function loadReaperOscSettings() {
    try {
      const [host, port] = await invoke("get_reaper_osc_settings");
      reaperOscHost.value = host;
      reaperOscPort.value = port;
    } catch (error) {
      reaperOscSettingsResult.textContent =
        `Could not load REAPER OSC settings: ${error}`;
    }
  }

  void loadReaperOscSettings();

  saveReaperOscSettingsButton.addEventListener("click", async () => {
    const host = reaperOscHost.value.trim();
    const port = Number(reaperOscPort.value);

    if (!host || !Number.isInteger(port) || port < 1 || port > 65535) {
      reaperOscSettingsResult.textContent =
        "Enter a host and a port between 1 and 65535";
      return;
    }

    saveReaperOscSettingsButton.disabled = true;
    reaperOscSettingsResult.textContent = "Saving REAPER OSC settings...";

    try {
      await invoke("save_reaper_osc_settings", { host, port });
      reaperOscSettingsResult.textContent = "REAPER OSC settings saved";
    } catch (error) {
      reaperOscSettingsResult.textContent =
        `Could not save REAPER OSC settings: ${error}`;
    } finally {
      saveReaperOscSettingsButton.disabled = false;
    }
  });

  testReaperConnectionButton.addEventListener("click", async () => {
    testReaperConnectionButton.disabled = true;
    reaperOscSettingsResult.textContent = "Testing REAPER connection...";

    try {
      const markerNumber = await invoke("test_reaper_osc_marker");
      reaperOscSettingsResult.textContent =
        `REAPER connection test marker created (marker ${markerNumber})`;
    } catch (error) {
      reaperOscSettingsResult.textContent = `REAPER connection test failed: ${error}`;
    } finally {
      testReaperConnectionButton.disabled = false;
    }
  });

  async function loadSceneFilePath() {
    try {
      const path = await invoke("get_scene_file_path");
      sceneFilePath.textContent = `Scene file: ${path}`;
    } catch (error) {
      sceneFilePath.textContent = `Could not load scene file path: ${error}`;
    }
  }

  void loadSceneFilePath();

  async function refreshMidiInputs() {
    midiButton.disabled = true;
    midiResult.textContent = "Refreshing MIDI inputs...";

    try {
      const inputs = await invoke("list_midi_inputs");
      const savedInput = await invoke("get_saved_midi_input");
      const prompt = document.createElement("option");
      prompt.value = "";
      prompt.textContent = inputs.length
        ? "Select MIDI input"
        : "No MIDI inputs found";
      midiInputSelect.replaceChildren(
        prompt,
        ...inputs.map((name) => {
          const option = document.createElement("option");
          option.value = name;
          option.textContent = name;
          return option;
        }),
      );
      midiInputSelect.disabled = !inputs.length;

      if (savedInput && inputs.includes(savedInput)) {
        midiInputSelect.value = savedInput;
        midiResult.textContent = `Saved MIDI input restored: ${savedInput}`;
      } else if (savedInput) {
        midiResult.textContent = "Saved MIDI input not found";
      } else {
        midiResult.textContent = inputs.length
          ? `MIDI inputs available: ${inputs.join(", ")}`
          : "No MIDI inputs available";
      }

      startMidiLogButton.disabled = dLiveLinkRunning || !midiInputSelect.value;
    } catch (error) {
      midiResult.textContent = `Could not refresh MIDI inputs: ${error}`;
    } finally {
      midiButton.disabled = false;
    }
  }

  void refreshMidiInputs();

  revealSceneFileButton.addEventListener("click", async () => {
    revealSceneFileButton.disabled = true;
    sceneFileRevealResult.textContent = "Opening scene file location...";

    try {
      await invoke("reveal_scene_file");
      sceneFileRevealResult.textContent = "Scene file location opened";
    } catch (error) {
      sceneFileRevealResult.textContent =
        `Could not open scene file location: ${error}`;
    } finally {
      revealSceneFileButton.disabled = false;
    }
  });

  reloadSceneNamesButton.addEventListener("click", async () => {
    reloadSceneNamesButton.disabled = true;
    sceneFileReloadResult.textContent = "Reloading scene names...";

    try {
      await invoke("reload_scene_names");
      sceneFileReloadResult.textContent = "Scene names reloaded";
    } catch (error) {
      sceneFileReloadResult.textContent = `Could not reload scene names: ${error}`;
    } finally {
      reloadSceneNamesButton.disabled = false;
    }
  });

  midiButton.addEventListener("click", refreshMidiInputs);

  midiInputSelect.addEventListener("change", async () => {
    const portName = midiInputSelect.value;
    startMidiLogButton.disabled = dLiveLinkRunning || !portName;

    if (!portName) {
      return;
    }

    try {
      await invoke("save_midi_input", { portName });
      midiResult.textContent = "MIDI input saved";
    } catch (error) {
      midiResult.textContent = `Could not save MIDI input: ${error}`;
    }
  });

  startMidiLogButton.addEventListener("click", async () => {
    if (dLiveLinkRunning) {
      midiLogResult.textContent = "dLive Link is already running";
      return;
    }

    const portName = midiInputSelect.value;

    if (!portName) {
      midiLogResult.textContent = "Select a MIDI input before starting dLive Link";
      return;
    }

    startMidiLogButton.disabled = true;
    midiLogResult.textContent = "Starting dLive Link...";
    midiSceneResult.textContent = "";
    midiSceneNameResult.textContent = "";
    midiLog.textContent = "";

    try {
      if (!unlistenMidiMessages) {
        unlistenMidiMessages = await window.__TAURI__.event.listen(
          "midi-message",
          (event) => {
            midiLog.textContent = `${event.payload}\n${midiLog.textContent}`;
          },
        );
        await window.__TAURI__.event.listen(
          "midi-scene-recall",
          (event) => {
            const [sceneNumber, sceneName] = event.payload;
            midiSceneResult.textContent = `Detected scene ${sceneNumber}`;
            midiSceneNameResult.textContent = `Scene name: ${sceneName}`;
          },
        );
      }

      await invoke("start_midi_log", { portName });
      dLiveLinkRunning = true;
      midiLogResult.textContent = "dLive Link running";
      stopMidiLogButton.disabled = false;
    } catch (error) {
      midiLogResult.textContent = `Could not start dLive Link: ${error}`;
    } finally {
      startMidiLogButton.disabled = dLiveLinkRunning;
    }
  });

  stopMidiLogButton.addEventListener("click", async () => {
    stopMidiLogButton.disabled = true;
    midiLogResult.textContent = "Stopping dLive Link...";

    try {
      await invoke("stop_midi_log");
      dLiveLinkRunning = false;
      startMidiLogButton.disabled = !midiInputSelect.value;
      midiLogResult.textContent = "dLive Link stopped";
    } catch (error) {
      midiLogResult.textContent = `Could not stop dLive Link: ${error}`;
    } finally {
      stopMidiLogButton.disabled = !dLiveLinkRunning;
    }
  });
});
