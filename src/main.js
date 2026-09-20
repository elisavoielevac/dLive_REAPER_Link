const { invoke } = window.__TAURI__.core;
const { open: openFileDialog, save: saveFileDialog } = window.__TAURI__.dialog;

window.addEventListener("DOMContentLoaded", () => {
  const appViews = [
    {
      button: document.querySelector("#tab-live"),
      panel: document.querySelector("#view-live"),
    },
    {
      button: document.querySelector("#tab-scene-lists"),
      panel: document.querySelector("#view-scene-lists"),
    },
    {
      button: document.querySelector("#tab-setup"),
      panel: document.querySelector("#view-setup"),
    },
    {
      button: document.querySelector("#tab-diagnostics"),
      panel: document.querySelector("#view-diagnostics"),
    },
  ];

  function showAppView(index, focusTab = false) {
    appViews.forEach((view, viewIndex) => {
      const selected = viewIndex === index;
      view.button.setAttribute("aria-selected", String(selected));
      view.panel.hidden = !selected;
    });

    if (focusTab) {
      appViews[index].button.focus();
    }
  }

  appViews.forEach((view, index) => {
    view.button.addEventListener("click", () => showAppView(index));
    view.button.addEventListener("keydown", (event) => {
      let nextIndex = index;

      if (event.key === "ArrowRight") {
        nextIndex = (index + 1) % appViews.length;
      } else if (event.key === "ArrowLeft") {
        nextIndex = (index - 1 + appViews.length) % appViews.length;
      } else if (event.key === "Home") {
        nextIndex = 0;
      } else if (event.key === "End") {
        nextIndex = appViews.length - 1;
      } else {
        return;
      }

      event.preventDefault();
      showAppView(nextIndex, true);
    });
  });

  const reaperOscHost = document.querySelector("#reaper-osc-host");
  const reaperOscPort = document.querySelector("#reaper-osc-port");
  const saveReaperOscSettingsButton = document.querySelector(
    "#save-reaper-osc-settings",
  );
  const testReaperConnectionButton = document.querySelector(
    "#test-reaper-connection",
  );
  const markerModeToggleButton = document.querySelector("#toggle-marker-mode");
  const markerModeToggleState = document.querySelector(
    "#marker-mode-toggle-state",
  );
  const markerModeResult = document.querySelector("#marker-mode-result");
  const reaperOscSettingsResult = document.querySelector(
    "#reaper-osc-settings-result",
  );
  const readinessMidiInput = document.querySelector("#readiness-midi-input");
  const readinessReaperOsc = document.querySelector("#readiness-reaper-osc");
  const readinessSceneNames = document.querySelector("#readiness-scene-names");
  const midiButton = document.querySelector("#list-midi-inputs");
  const midiResult = document.querySelector("#midi-inputs-result");
  const sceneFilePath = document.querySelector("#scene-file-path");
  const sceneFileLocation = document.querySelector("#scene-file-location");
  const revealSceneFileButton = document.querySelector("#reveal-scene-file");
  const reloadSceneNamesButton = document.querySelector("#reload-scene-names");
  const sceneFileRevealResult = document.querySelector(
    "#scene-file-reveal-result",
  );
  const sceneFileReloadResult = document.querySelector(
    "#scene-file-reload-result",
  );
  const openSceneEditorButton = document.querySelector("#open-scene-editor");
  const exportSceneListButton = document.querySelector("#export-scene-list");
  const importSceneListButton = document.querySelector("#import-scene-list");
  const sceneListTransferResult = document.querySelector(
    "#scene-list-transfer-result",
  );
  const createSceneListButton = document.querySelector("#create-scene-list");
  const saveSceneListBackupButton = document.querySelector(
    "#save-scene-list-backup",
  );
  const sceneListBackups = document.querySelector("#scene-list-backups");
  const refreshSceneListBackupsButton = document.querySelector(
    "#refresh-scene-list-backups",
  );
  const restoreSceneListBackupButton = document.querySelector(
    "#restore-scene-list-backup",
  );
  const sceneListBackupResult = document.querySelector(
    "#scene-list-backup-result",
  );
  const sceneListNamePrompt = document.querySelector("#scene-list-name-prompt");
  const sceneListNamePromptHeading = document.querySelector(
    "#scene-list-name-prompt-heading",
  );
  const sceneListNamePromptInput = document.querySelector(
    "#scene-list-name-prompt-input",
  );
  const sceneListNamePromptResult = document.querySelector(
    "#scene-list-name-prompt-result",
  );
  const cancelSceneListNamePromptButton = document.querySelector(
    "#cancel-scene-list-name-prompt",
  );
  const confirmSceneListNamePromptButton = document.querySelector(
    "#confirm-scene-list-name-prompt",
  );
  const importSceneListConfirmation = document.querySelector(
    "#import-scene-list-confirmation",
  );
  const importSceneListPath = document.querySelector("#import-scene-list-path");
  const cancelSceneListImportButton = document.querySelector(
    "#cancel-scene-list-import",
  );
  const confirmSceneListImportButton = document.querySelector(
    "#confirm-scene-list-import",
  );
  const restoreSceneListBackupConfirmation = document.querySelector(
    "#restore-scene-list-backup-confirmation",
  );
  const restoreSceneListBackupName = document.querySelector(
    "#restore-scene-list-backup-name",
  );
  const cancelSceneListBackupRestoreButton = document.querySelector(
    "#cancel-scene-list-backup-restore",
  );
  const confirmSceneListBackupRestoreButton = document.querySelector(
    "#confirm-scene-list-backup-restore",
  );
  const sceneEditorPanel = document.querySelector("#scene-editor-panel");
  const closeSceneEditorButton = document.querySelector("#close-scene-editor");
  const discardSceneEditorConfirmation = document.querySelector(
    "#discard-scene-editor-confirmation",
  );
  const keepEditingSceneNamesButton = document.querySelector(
    "#keep-editing-scene-names",
  );
  const discardSceneNameChangesButton = document.querySelector(
    "#discard-scene-name-changes",
  );
  const sceneEditorFilePath = document.querySelector("#scene-editor-file-path");
  const sceneEditorHeading = document.querySelector("#scene-editor-heading");
  const sceneEditorFileLocation = document.querySelector(
    "#scene-editor-file-location",
  );
  const sceneEditor = document.querySelector("#scene-editor");
  const saveSceneNamesButton = document.querySelector("#save-scene-names");
  const sceneEditorSaveAsName = document.querySelector(
    "#scene-editor-save-as-name",
  );
  const saveSceneListAsButton = document.querySelector(
    "#save-scene-list-as",
  );
  const createSceneListFromEditorButton = document.querySelector(
    "#create-scene-list-from-editor",
  );
  const sceneEditorResult = document.querySelector("#scene-editor-result");
  const midiInputSelect = document.querySelector("#midi-input-select");
  const midiListenerToggleButton = document.querySelector(
    "#toggle-midi-listener",
  );
  const midiListenerToggleState = document.querySelector(
    "#midi-listener-toggle-state",
  );
  const midiLogResult = document.querySelector("#midi-log-result");
  const midiSceneResult = document.querySelector("#midi-scene-result");
  const midiSceneNameResult = document.querySelector("#midi-scene-name-result");
  const midiSceneDiagnostics = document.querySelector(
    "#midi-scene-diagnostics",
  );
  const markerDecision = document.querySelector("#marker-decision");
  const midiLog = document.querySelector("#midi-log");
  let dLiveLinkRunning = false;
  let markerMode = "always";
  let savedReaperOscTarget;
  let sceneNamesCount;
  let loadedSceneListName;
  let sceneListFilePath;
  let sceneEditorDirty = false;
  let pendingSceneListImportPath;
  let pendingSceneListBackupName;
  let pendingSceneListNameAction;
  let unlistenMidiMessages;

  function updateReadiness() {
    readinessMidiInput.textContent = `MIDI Input: ${
      midiInputSelect.value ? "Ready" : "Not selected"
    }`;
    readinessReaperOsc.textContent = savedReaperOscTarget
      ? `REAPER OSC: Configured (${savedReaperOscTarget.host}:${savedReaperOscTarget.port})`
      : "REAPER OSC: Check settings";
    readinessSceneNames.textContent = loadedSceneListName
      ? loadedSceneListName +
        (Number.isInteger(sceneNamesCount)
          ? " (" + sceneNamesCount + " scenes)"
          : "")
      : "No scene list loaded";
  }

  function updateMidiListenerToggle() {
    const state = dLiveLinkRunning
      ? "Stop Listening"
      : "Start Listening";
    midiListenerToggleState.textContent = state;
    midiListenerToggleButton.setAttribute(
      "aria-label",
      `dLive Listener: ${state}`,
    );
    midiListenerToggleButton.classList.toggle("is-listening", dLiveLinkRunning);
    midiListenerToggleButton.disabled =
      !dLiveLinkRunning && !midiInputSelect.value;
  }

  function updateMarkerModeToggle() {
    const state = markerMode === "always" ? "Always" : "Recording Only";
    markerModeToggleState.textContent = state;
    markerModeToggleButton.setAttribute("aria-label", `Marker Mode: ${state}`);
    markerModeToggleButton.classList.toggle(
      "is-recording-only",
      markerMode === "recording_only",
    );
    markerModeToggleButton.classList.toggle("is-always", markerMode === "always");
  }

  function updateSceneListDisplay() {
    const loadedSceneListLabel = loadedSceneListName || "No scene list loaded";
    sceneFilePath.textContent =
      "Loaded Scene List: " +
      loadedSceneListLabel;
    sceneEditorHeading.textContent = "Editing: " + loadedSceneListLabel;
    sceneEditorFilePath.textContent =
      loadedSceneListName ? "Loaded Scene List" : "No scene list loaded";
    sceneFileLocation.textContent = sceneListFilePath
      ? "File location: " + sceneListFilePath
      : "";
    sceneEditorFileLocation.textContent = sceneListFilePath
      ? "File location: " + sceneListFilePath
      : "";
    openSceneEditorButton.disabled = !loadedSceneListName;
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

  async function loadMarkerMode() {
    try {
      markerMode = await invoke("get_marker_mode");
      updateMarkerModeToggle();
    } catch (error) {
      markerModeResult.textContent = `Could not load marker mode: ${error}`;
      markerModeResult.hidden = false;
    } finally {
      markerModeToggleButton.disabled = false;
    }
  }

  void loadMarkerMode();

  markerModeToggleButton.addEventListener("click", async () => {
    const nextMode =
      markerMode === "always" ? "recording_only" : "always";
    markerModeToggleButton.disabled = true;
    try {
      await invoke("save_marker_mode", { mode: nextMode });
      markerMode = nextMode;
      updateMarkerModeToggle();
      markerModeResult.hidden = true;
    } catch (error) {
      markerModeResult.textContent = `Could not save marker mode: ${error}`;
      markerModeResult.hidden = false;
      await loadMarkerMode();
    } finally {
      markerModeToggleButton.disabled = false;
    }
  });

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
      sceneListFilePath = await invoke("get_scene_file_path");
      updateSceneListDisplay();
    } catch (error) {
      sceneFilePath.textContent = `Could not load scene list location: ${error}`;
      sceneFileLocation.textContent = "";
      sceneEditorFilePath.textContent = "Could not load scene list location";
      sceneEditorFileLocation.textContent = "";
    }
  }

  void loadSceneFilePath();

  async function loadLoadedSceneList() {
    try {
      loadedSceneListName = await invoke("get_loaded_scene_list");
    } catch (error) {
      sceneListTransferResult.textContent =
        `Could not load saved scene list selection: ${error}`;
    } finally {
      updateSceneListDisplay();
      updateReadiness();
      await refreshSceneListBackups(false);
    }
  }

  void loadLoadedSceneList();

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
    saveSceneListAsButton.disabled = true;
    createSceneListFromEditorButton.disabled = true;
    sceneEditorResult.textContent = "Loading scene list...";

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
      saveSceneListAsButton.disabled = false;
      createSceneListFromEditorButton.disabled = false;
    } catch (error) {
      sceneEditorResult.textContent = `Could not load scene list: ${error}`;
    }
  }

  async function openSceneEditor() {
    if (!loadedSceneListName) {
      sceneListBackupResult.textContent =
        "Create, load, or import a list before editing";
      return;
    }
    sceneEditorPanel.hidden = false;
    document.body.classList.add("scene-editor-open");
    await loadSceneFilePath();
    await loadSceneEditor();
  }

  function closeSceneEditorNow() {
    discardSceneEditorConfirmation.hidden = true;
    sceneEditorPanel.hidden = true;
    document.body.classList.remove("scene-editor-open");
    sceneEditor.replaceChildren();
    sceneEditorResult.textContent = "";
    sceneEditorSaveAsName.value = "";
    sceneEditorDirty = false;
  }

  function closeSceneEditor() {
    if (sceneEditorDirty) {
      discardSceneEditorConfirmation.hidden = false;
      discardSceneNameChangesButton.focus();
      return;
    }

    closeSceneEditorNow();
  }

  function closeSceneListImportConfirmation() {
    importSceneListConfirmation.hidden = true;
    document.body.classList.remove("scene-list-import-open");
    pendingSceneListImportPath = undefined;
  }

  function openSceneListNamePrompt(action) {
    pendingSceneListNameAction = action;
    sceneListNamePromptHeading.textContent =
      action === "create" ? "Create New" : "Save As";
    confirmSceneListNamePromptButton.textContent =
      action === "create" ? "Create New" : "Save As";
    sceneListNamePromptInput.value = "";
    sceneListNamePromptResult.textContent = "";
    sceneListNamePrompt.hidden = false;
    document.body.classList.add("scene-list-name-prompt-open");
    sceneListNamePromptInput.focus();
  }

  function closeSceneListNamePrompt() {
    sceneListNamePrompt.hidden = true;
    document.body.classList.remove("scene-list-name-prompt-open");
    pendingSceneListNameAction = undefined;
    sceneListNamePromptInput.value = "";
    sceneListNamePromptResult.textContent = "";
  }

  async function exportSceneList() {
    exportSceneListButton.disabled = true;
    sceneListTransferResult.textContent = "Choose export location...";

    try {
      const destinationPath = await saveFileDialog({
        title: "Export Scene List",
        defaultPath: "dLive_Scenes_export.txt",
        filters: [{ name: "Text files", extensions: ["txt"] }],
      });

      if (!destinationPath) {
        sceneListTransferResult.textContent = "Export cancelled; loaded scene list unchanged";
        return;
      }

      await invoke("export_scene_list", { destinationPath });
      sceneListTransferResult.textContent = "Scene list exported; loaded scene list unchanged";
    } catch (error) {
      sceneListTransferResult.textContent = `Could not export a copy of the loaded scene list: ${error}`;
    } finally {
      exportSceneListButton.disabled = false;
    }
  }

  async function selectSceneListImport() {
    if (!sceneEditorPanel.hidden && sceneEditorDirty) {
      sceneEditorResult.textContent =
        "Save or discard your scene list edits before loading another list";
      discardSceneEditorConfirmation.hidden = false;
      return;
    }
    importSceneListButton.disabled = true;
    sceneListTransferResult.textContent = "Choose a .txt file to import...";

    try {
      const sourcePath = await openFileDialog({
        title: "Import Scene List",
        multiple: false,
        directory: false,
        filters: [{ name: "Text files", extensions: ["txt"] }],
      });

      if (!sourcePath || Array.isArray(sourcePath)) {
        sceneListTransferResult.textContent = "Import cancelled; loaded scene list unchanged";
        return;
      }

      pendingSceneListImportPath = sourcePath;
      importSceneListPath.textContent = `External file: ${sourcePath}`;
      importSceneListConfirmation.hidden = false;
      document.body.classList.add("scene-list-import-open");
      confirmSceneListImportButton.focus();
    } catch (error) {
      sceneListTransferResult.textContent = `Could not open an external scene list: ${error}`;
    } finally {
      importSceneListButton.disabled = false;
    }
  }

  async function importSceneList() {
    if (!pendingSceneListImportPath) {
      return;
    }

    cancelSceneListImportButton.disabled = true;
    confirmSceneListImportButton.disabled = true;
    sceneListTransferResult.textContent = "Importing and loading scene list...";

    try {
      const [sceneListName, count] = await invoke("import_scene_list", {
        sourcePath: pendingSceneListImportPath,
      });
      loadedSceneListName = sceneListName;
      sceneNamesCount = count;
      updateSceneListDisplay();
      updateReadiness();
      sceneListTransferResult.textContent = `Scene list imported and loaded: ${sceneListName}`;
      closeSceneListImportConfirmation();
      await refreshSceneListBackups(false);

      if (!sceneEditorPanel.hidden) {
        await loadSceneEditor();
      }
    } catch (error) {
      sceneListTransferResult.textContent = `Could not import scene list: ${error}`;
    } finally {
      cancelSceneListImportButton.disabled = false;
      confirmSceneListImportButton.disabled = false;
    }
  }

  async function refreshSceneListBackups(showStatus = true) {
    refreshSceneListBackupsButton.disabled = true;

    try {
      const backups = await invoke("list_saved_scene_lists");
      const prompt = document.createElement("option");
      prompt.value = "";
      prompt.textContent = backups.length
        ? "Select a list"
        : "No saved lists";
      sceneListBackups.replaceChildren(
        prompt,
        ...backups.map((backupName) => {
          const option = document.createElement("option");
          option.value = backupName;
          option.textContent = backupName;
          return option;
        }),
      );
      sceneListBackups.disabled = !backups.length;
      sceneListBackups.value = backups.includes(loadedSceneListName)
        ? loadedSceneListName
        : "";
      restoreSceneListBackupButton.disabled = !sceneListBackups.value;

      if (showStatus) {
        sceneListBackupResult.textContent = backups.length
          ? `${backups.length} saved list${backups.length === 1 ? "" : "s"} available`
          : "No saved lists";
      }
    } catch (error) {
      sceneListBackups.disabled = true;
      restoreSceneListBackupButton.disabled = true;
      sceneListBackupResult.textContent = `Could not load saved scene lists: ${error}`;
    } finally {
      refreshSceneListBackupsButton.disabled = false;
    }
  }

  async function saveSceneListBackup(name) {
    if (!sceneEditorPanel.hidden && sceneEditorDirty) {
      sceneEditorResult.textContent =
        "Save or discard your scene list edits before saving another list";
      discardSceneEditorConfirmation.hidden = false;
      return false;
    }
    saveSceneListBackupButton.disabled = true;
    sceneListBackupResult.textContent = "Saving scene list...";

    try {
      const backupName = await invoke("save_scene_list_as", {
        name,
      });
      loadedSceneListName = backupName;
      sceneNamesCount = await invoke("get_scene_names_count");
      updateSceneListDisplay();
      updateReadiness();
      await refreshSceneListBackups(false);
      sceneListBackups.value = backupName;
      restoreSceneListBackupButton.disabled = false;
      sceneListBackupResult.textContent = `Scene list saved and loaded: ${backupName}`;
      return true;
    } catch (error) {
      sceneListBackupResult.textContent = `Could not save scene list: ${error}`;
      sceneListNamePromptResult.textContent = `Could not save: ${error}`;
      return false;
    } finally {
      saveSceneListBackupButton.disabled = false;
    }
  }

  async function createSceneList(name, result) {
    if (!name) {
      result.textContent = "Enter a name for the new scene list";
      return false;
    }

    result.textContent = "Creating new scene list...";
    try {
      const [sceneListName, count] = await invoke("create_scene_list", {
        name,
      });
      loadedSceneListName = sceneListName;
      sceneNamesCount = count;
      updateSceneListDisplay();
      updateReadiness();
      await refreshSceneListBackups(false);
      sceneListBackups.value = sceneListName;
      restoreSceneListBackupButton.disabled = false;
      result.textContent = "New scene list created and loaded: " + sceneListName;

      if (!sceneEditorPanel.hidden) {
        await loadSceneEditor();
      }
      return true;
    } catch (error) {
      result.textContent = `Could not create scene list: ${error}`;
      return false;
    }
  }

  function closeSceneListBackupRestoreConfirmation() {
    restoreSceneListBackupConfirmation.hidden = true;
    document.body.classList.remove("scene-list-backup-restore-open");
    pendingSceneListBackupName = undefined;
  }

  function requestSceneListBackupRestore() {
    const backupName = sceneListBackups.value;
    if (!backupName) {
      sceneListBackupResult.textContent = "Select a list to load";
      return;
    }

    if (!sceneEditorPanel.hidden && sceneEditorDirty) {
      sceneEditorResult.textContent =
        "Save or discard your scene list edits before loading another list";
      discardSceneEditorConfirmation.hidden = false;
      return;
    }

    pendingSceneListBackupName = backupName;
    restoreSceneListBackupName.textContent = `Scene list to load: ${backupName}`;
    restoreSceneListBackupConfirmation.hidden = false;
    document.body.classList.add("scene-list-backup-restore-open");
    confirmSceneListBackupRestoreButton.focus();
  }

  async function restoreSceneListBackup() {
    if (!pendingSceneListBackupName) {
      return;
    }

    cancelSceneListBackupRestoreButton.disabled = true;
    confirmSceneListBackupRestoreButton.disabled = true;
    sceneListBackupResult.textContent = "Loading selected scene list...";

    try {
      sceneNamesCount = await invoke("load_scene_list", {
        sceneListName: pendingSceneListBackupName,
      });
      loadedSceneListName = pendingSceneListBackupName;
      updateSceneListDisplay();
      updateReadiness();
      sceneListBackupResult.textContent = `Scene list loaded: ${loadedSceneListName}`;
      closeSceneListBackupRestoreConfirmation();

      if (!sceneEditorPanel.hidden) {
        await loadSceneEditor();
      }
    } catch (error) {
      sceneListBackupResult.textContent = `Could not load selected scene list: ${error}`;
    } finally {
      cancelSceneListBackupRestoreButton.disabled = false;
      confirmSceneListBackupRestoreButton.disabled = false;
    }
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
          ? "Choose a MIDI input."
          : "No MIDI inputs available";
      }

      updateMidiListenerToggle();
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
    sceneFileRevealResult.textContent = "Opening scene list file location...";

    try {
      await invoke("reveal_scene_file");
      sceneFileRevealResult.textContent = "Scene list file location opened";
    } catch (error) {
      sceneFileRevealResult.textContent =
        `Could not open scene list file location: ${error}`;
    } finally {
      revealSceneFileButton.disabled = false;
    }
  });

  reloadSceneNamesButton.addEventListener("click", async () => {
    reloadSceneNamesButton.disabled = true;
    sceneFileReloadResult.textContent = "Reloading scene list...";

    try {
      sceneNamesCount = await invoke("reload_scene_names");
      updateReadiness();
      sceneFileReloadResult.textContent = "Scene list reloaded";
      if (!sceneEditorPanel.hidden) {
        await loadSceneEditor();
      }
    } catch (error) {
      sceneFileReloadResult.textContent = `Could not reload scene list: ${error}`;
    } finally {
      reloadSceneNamesButton.disabled = false;
    }
  });

  saveSceneNamesButton.addEventListener("click", async () => {
    const names = Array.from(sceneEditor.querySelectorAll("input"), (input) =>
      input.value,
    );

    if (names.length !== 500) {
      sceneEditorResult.textContent = "Could not save scene list: 500 scene slots are required";
      return;
    }

    saveSceneNamesButton.disabled = true;
    closeSceneEditorButton.disabled = true;
    sceneEditorResult.textContent = "Saving scene list...";

    try {
      sceneNamesCount = await invoke("save_scene_names", { names });
      updateReadiness();
      sceneEditorDirty = false;
      sceneEditorResult.textContent = "Scene list saved and reloaded";
    } catch (error) {
      sceneEditorResult.textContent = `Could not save scene list: ${error}`;
    } finally {
      saveSceneNamesButton.disabled = false;
      closeSceneEditorButton.disabled = false;
    }
  });

  saveSceneListAsButton.addEventListener("click", async () => {
    const name = sceneEditorSaveAsName.value.trim();
    const names = Array.from(sceneEditor.querySelectorAll("input"), (input) =>
      input.value,
    );

    if (!name) {
      sceneEditorResult.textContent = "Enter a name for the new scene list";
      return;
    }
    if (names.length !== 500) {
      sceneEditorResult.textContent =
        "Could not save scene list: 500 scene slots are required";
      return;
    }

    saveSceneNamesButton.disabled = true;
    saveSceneListAsButton.disabled = true;
    closeSceneEditorButton.disabled = true;
    sceneEditorResult.textContent = "Saving new scene list...";

    try {
      const sceneListName = await invoke("save_scene_list_as", { name, names });
      sceneNamesCount = await invoke("get_scene_names_count");
      loadedSceneListName = sceneListName;
      sceneEditorSaveAsName.value = "";
      sceneEditorDirty = false;
      updateSceneListDisplay();
      updateReadiness();
      await refreshSceneListBackups(false);
      sceneListBackups.value = sceneListName;
      restoreSceneListBackupButton.disabled = false;
      sceneEditorResult.textContent = `Scene list saved and loaded: ${sceneListName}`;
    } catch (error) {
      sceneEditorResult.textContent = `Could not save scene list: ${error}`;
    } finally {
      saveSceneNamesButton.disabled = false;
      saveSceneListAsButton.disabled = false;
      closeSceneEditorButton.disabled = false;
    }
  });

  createSceneListButton.addEventListener("click", async () => {
    openSceneListNamePrompt("create");
  });

  createSceneListFromEditorButton.addEventListener("click", async () => {
    if (sceneEditorDirty) {
      sceneEditorResult.textContent =
        "Save or discard your edits before creating a new scene list";
      return;
    }

    createSceneListFromEditorButton.disabled = true;
    const created = await createSceneList(
      sceneEditorSaveAsName.value.trim(),
      sceneEditorResult,
    );
    if (created) {
      sceneEditorSaveAsName.value = "";
    }
    createSceneListFromEditorButton.disabled = false;
  });

  openSceneEditorButton.addEventListener("click", () => {
    void openSceneEditor();
  });

  void refreshSceneListBackups();

  exportSceneListButton.addEventListener("click", () => {
    void exportSceneList();
  });

  importSceneListButton.addEventListener("click", () => {
    void selectSceneListImport();
  });

  cancelSceneListImportButton.addEventListener("click", () => {
    closeSceneListImportConfirmation();
    sceneListTransferResult.textContent = "Import cancelled; loaded scene list unchanged";
  });

  confirmSceneListImportButton.addEventListener("click", () => {
    void importSceneList();
  });

  saveSceneListBackupButton.addEventListener("click", () => {
    openSceneListNamePrompt("save_as");
  });

  cancelSceneListNamePromptButton.addEventListener("click", () => {
    closeSceneListNamePrompt();
    sceneListBackupResult.textContent = "Name cancelled";
  });

  confirmSceneListNamePromptButton.addEventListener("click", async () => {
    const name = sceneListNamePromptInput.value.trim();
    if (!name) {
      sceneListNamePromptResult.textContent = "Enter a name";
      return;
    }

    confirmSceneListNamePromptButton.disabled = true;
    cancelSceneListNamePromptButton.disabled = true;

    try {
      let completed = false;
      if (pendingSceneListNameAction === "create") {
        completed = await createSceneList(name, sceneListNamePromptResult);
      } else if (pendingSceneListNameAction === "save_as") {
        completed = await saveSceneListBackup(name);
      }

      if (completed) {
        closeSceneListNamePrompt();
      }
    } finally {
      confirmSceneListNamePromptButton.disabled = false;
      cancelSceneListNamePromptButton.disabled = false;
    }
  });

  sceneListNamePromptInput.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      confirmSceneListNamePromptButton.click();
    } else if (event.key === "Escape") {
      closeSceneListNamePrompt();
      sceneListBackupResult.textContent = "Name cancelled";
    }
  });

  refreshSceneListBackupsButton.addEventListener("click", () => {
    void refreshSceneListBackups();
  });

  sceneListBackups.addEventListener("change", () => {
    restoreSceneListBackupButton.disabled = !sceneListBackups.value;
  });

  restoreSceneListBackupButton.addEventListener(
    "click",
    requestSceneListBackupRestore,
  );

  cancelSceneListBackupRestoreButton.addEventListener("click", () => {
    closeSceneListBackupRestoreConfirmation();
    sceneListBackupResult.textContent = "Load cancelled; loaded scene list unchanged";
  });

  confirmSceneListBackupRestoreButton.addEventListener("click", () => {
    void restoreSceneListBackup();
  });

  closeSceneEditorButton.addEventListener("click", closeSceneEditor);

  keepEditingSceneNamesButton.addEventListener("click", () => {
    discardSceneEditorConfirmation.hidden = true;
    closeSceneEditorButton.focus();
  });

  discardSceneNameChangesButton.addEventListener("click", closeSceneEditorNow);

  sceneEditor.addEventListener("input", () => {
    sceneEditorDirty = true;
  });

  midiButton.addEventListener("click", refreshMidiInputs);

  midiInputSelect.addEventListener("change", async () => {
    const portName = midiInputSelect.value;
    updateMidiListenerToggle();
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

  async function startMidiListener() {
    if (dLiveLinkRunning) {
      midiLogResult.textContent = "dLive Listener is already running";
      return;
    }

    const portName = midiInputSelect.value;

    if (!portName) {
      midiLogResult.textContent = "Select a MIDI input before starting the dLive Listener";
      return;
    }

    midiListenerToggleButton.disabled = true;
    midiLogResult.textContent = "Starting dLive Listener...";
    midiSceneResult.textContent = "";
    midiSceneNameResult.textContent = "";
    midiSceneDiagnostics.textContent = "";
    markerDecision.textContent = "";
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
            const [sceneNumber, sceneName, program, channel, bank, bankLsb] =
              event.payload;
            midiSceneResult.textContent = `Detected scene ${sceneNumber}`;
            midiSceneNameResult.textContent = `Scene name: ${sceneName}`;
            midiSceneDiagnostics.textContent = `Scene recall: raw Program Change ${program}, MIDI channel ${channel}, bank ${bank} (LSB ${bankLsb}), decoded scene ${sceneNumber}, ${sceneName}`;
          },
        );
        await window.__TAURI__.event.listen("marker-decision", (event) => {
          markerDecision.textContent = event.payload;
        });
      }

      await invoke("start_midi_log", { portName });
      dLiveLinkRunning = true;
      updateReadiness();
      midiLogResult.textContent = "dLive Listener is listening for scene recalls";
    } catch (error) {
      midiLogResult.textContent = `Could not start dLive Listener: ${error}`;
    } finally {
      updateMidiListenerToggle();
    }
  }

  async function stopMidiListener() {
    midiListenerToggleButton.disabled = true;
    midiLogResult.textContent = "Stopping dLive Listener...";

    try {
      await invoke("stop_midi_log");
      dLiveLinkRunning = false;
      updateReadiness();
      midiLogResult.textContent = "dLive Listener stopped";
    } catch (error) {
      midiLogResult.textContent = `Could not stop dLive Listener: ${error}`;
    } finally {
      updateMidiListenerToggle();
    }
  }

  midiListenerToggleButton.addEventListener("click", () => {
    if (dLiveLinkRunning) {
      void stopMidiListener();
    } else {
      void startMidiListener();
    }
  });
});
