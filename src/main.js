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
  const readinessMidiInput = document.querySelector("#readiness-midi-input");
  const readinessDLiveLink = document.querySelector("#readiness-dlive-link");
  const readinessReaperOsc = document.querySelector("#readiness-reaper-osc");
  const readinessSceneNames = document.querySelector("#readiness-scene-names");
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
  const openSceneEditorButton = document.querySelector("#open-scene-editor");
  const sceneEditorPanel = document.querySelector("#scene-editor-panel");
  const closeSceneEditorButton = document.querySelector("#close-scene-editor");
  const sceneEditorFilePath = document.querySelector("#scene-editor-file-path");
  const sceneEditor = document.querySelector("#scene-editor");
  const saveSceneNamesButton = document.querySelector("#save-scene-names");
  const sceneEditorResult = document.querySelector("#scene-editor-result");
  const midiInputSelect = document.querySelector("#midi-input-select");
  const startMidiLogButton = document.querySelector("#start-midi-log");
  const stopMidiLogButton = document.querySelector("#stop-midi-log");
  const midiLogResult = document.querySelector("#midi-log-result");
  const midiSceneResult = document.querySelector("#midi-scene-result");
  const midiSceneNameResult = document.querySelector("#midi-scene-name-result");
  const midiLog = document.querySelector("#midi-log");
  let dLiveLinkRunning = false;
  let savedReaperOscTarget;
  let sceneNamesCount;
  let sceneEditorDirty = false;
  let unlistenMidiMessages;

  function updateReadiness() {
    readinessMidiInput.textContent = `MIDI Input: ${
      midiInputSelect.value ? "Ready" : "Not selected"
    }`;
    readinessDLiveLink.textContent =
      `dLive Link: ${dLiveLinkRunning ? "Running" : "Stopped"}`;
    readinessReaperOsc.textContent = savedReaperOscTarget
      ? `REAPER OSC: Configured (${savedReaperOscTarget.host}:${savedReaperOscTarget.port})`
      : "REAPER OSC: Check settings";
    readinessSceneNames.textContent = Number.isInteger(sceneNamesCount)
      ? `Scene Names: ${sceneNamesCount} loaded`
      : "Scene Names: Not loaded";
  }

  updateReadiness();

  async function loadReaperOscSettings() {
    try {
      const [host, port] = await invoke("get_reaper_osc_settings");
      reaperOscHost.value = host;
      reaperOscPort.value = port;
      savedReaperOscTarget = { host, port };
      updateReadiness();
    } catch (error) {
      reaperOscSettingsResult.textContent =
        `Could not load REAPER OSC settings: ${error}`;
      updateReadiness();
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
      savedReaperOscTarget = { host, port };
      updateReadiness();
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
      sceneEditorFilePath.textContent = `Editing: ${path}`;
    } catch (error) {
      sceneFilePath.textContent = `Could not load scene file path: ${error}`;
      sceneEditorFilePath.textContent = "Could not load active scene file path";
    }
  }

  void loadSceneFilePath();

  async function loadSceneNamesStatus() {
    try {
      sceneNamesCount = await invoke("get_scene_names_count");
    } catch {
      sceneNamesCount = undefined;
    } finally {
      updateReadiness();
    }
  }

  void loadSceneNamesStatus();

  async function loadSceneEditor() {
    saveSceneNamesButton.disabled = true;
    sceneEditorResult.textContent = "Loading scene names...";

    try {
      const names = await invoke("get_scene_name_entries");
      const rows = document.createDocumentFragment();

      names.forEach((name, index) => {
        const sceneNumber = index + 1;
        const row = document.createElement("label");
        row.className = "scene-editor-row";

        const number = document.createElement("span");
        number.className = "scene-editor-number";
        number.textContent = `Scene ${sceneNumber}`;

        const input = document.createElement("input");
        input.type = "text";
        input.value = name;
        input.autocomplete = "off";
        input.spellcheck = false;
        input.setAttribute("aria-label", `Scene ${sceneNumber} name`);

        row.append(number, input);
        rows.append(row);
      });

      sceneEditor.replaceChildren(rows);
      sceneEditorDirty = false;
      sceneEditorResult.textContent = "";
      saveSceneNamesButton.disabled = false;
    } catch (error) {
      sceneEditorResult.textContent = `Could not load scene names: ${error}`;
    }
  }

  async function openSceneEditor() {
    sceneEditorPanel.hidden = false;
    document.body.classList.add("scene-editor-open");
    await loadSceneFilePath();
    await loadSceneEditor();
  }

  function closeSceneEditor() {
    if (
      sceneEditorDirty &&
      !window.confirm("Discard unsaved scene name changes?")
    ) {
      return;
    }

    sceneEditorPanel.hidden = true;
    document.body.classList.remove("scene-editor-open");
    sceneEditor.replaceChildren();
    sceneEditorResult.textContent = "";
    sceneEditorDirty = false;
  }

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
          ? "Choose your Allen & Heath MIDI TCP input, then start dLive Link."
          : "No MIDI inputs available";
      }

      startMidiLogButton.disabled = dLiveLinkRunning || !midiInputSelect.value;
      updateReadiness();
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
      sceneNamesCount = await invoke("reload_scene_names");
      updateReadiness();
      sceneFileReloadResult.textContent = "Scene names reloaded";
      if (!sceneEditorPanel.hidden) {
        await loadSceneEditor();
      }
    } catch (error) {
      sceneFileReloadResult.textContent = `Could not reload scene names: ${error}`;
    } finally {
      reloadSceneNamesButton.disabled = false;
    }
  });

  saveSceneNamesButton.addEventListener("click", async () => {
    const names = Array.from(sceneEditor.querySelectorAll("input"), (input) =>
      input.value,
    );

    if (names.length !== 500) {
      sceneEditorResult.textContent = "Could not save scene names: 500 scene slots are required";
      return;
    }

    saveSceneNamesButton.disabled = true;
    closeSceneEditorButton.disabled = true;
    sceneEditorResult.textContent = "Saving 500 scene names...";

    try {
      sceneNamesCount = await invoke("save_scene_names", { names });
      updateReadiness();
      sceneEditorDirty = false;
      sceneEditorResult.textContent = "Scene names saved and reloaded";
    } catch (error) {
      sceneEditorResult.textContent = `Could not save scene names: ${error}`;
    } finally {
      saveSceneNamesButton.disabled = false;
      closeSceneEditorButton.disabled = false;
    }
  });

  openSceneEditorButton.addEventListener("click", () => {
    void openSceneEditor();
  });

  closeSceneEditorButton.addEventListener("click", closeSceneEditor);

  sceneEditor.addEventListener("input", () => {
    sceneEditorDirty = true;
  });

  midiButton.addEventListener("click", refreshMidiInputs);

  midiInputSelect.addEventListener("change", async () => {
    const portName = midiInputSelect.value;
    startMidiLogButton.disabled = dLiveLinkRunning || !portName;
    updateReadiness();

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
      updateReadiness();
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
      updateReadiness();
      startMidiLogButton.disabled = !midiInputSelect.value;
      midiLogResult.textContent = "dLive Link stopped";
    } catch (error) {
      midiLogResult.textContent = `Could not stop dLive Link: ${error}`;
    } finally {
      stopMidiLogButton.disabled = !dLiveLinkRunning;
    }
  });
});
