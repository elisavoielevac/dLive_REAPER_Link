const { invoke } = window.__TAURI__.core;
const { open: openFileDialog, save: saveFileDialog } = window.__TAURI__.dialog;

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
  const sceneBackupName = document.querySelector("#scene-backup-name");
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
  const sceneEditorFileLocation = document.querySelector(
    "#scene-editor-file-location",
  );
  const sceneEditor = document.querySelector("#scene-editor");
  const saveSceneNamesButton = document.querySelector("#save-scene-names");
  const sceneEditorResult = document.querySelector("#scene-editor-result");
  const midiInputSelect = document.querySelector("#midi-input-select");
  const startMidiLogButton = document.querySelector("#start-midi-log");
  const stopMidiLogButton = document.querySelector("#stop-midi-log");
  const midiLogResult = document.querySelector("#midi-log-result");
  const midiSceneResult = document.querySelector("#midi-scene-result");
  const midiSceneNameResult = document.querySelector("#midi-scene-name-result");
  const midiSceneDiagnostics = document.querySelector(
    "#midi-scene-diagnostics",
  );
  const midiLog = document.querySelector("#midi-log");
  let dLiveLinkRunning = false;
  let savedReaperOscTarget;
  let sceneNamesCount;
  let sceneEditorDirty = false;
  let pendingSceneListImportPath;
  let pendingSceneListBackupName;
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
      ? `Loaded Scene List: Default Scene List (${sceneNamesCount} names loaded)`
      : "Loaded Scene List: Default Scene List (not loaded)";
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
      sceneFilePath.textContent =
        "Loaded Scene List: Default Scene List (used for marker names)";
      sceneFileLocation.textContent = `File location: ${path}`;
      sceneEditorFilePath.textContent =
        "Editing loaded scene list: Default Scene List";
      sceneEditorFileLocation.textContent = `File location: ${path}`;
    } catch (error) {
      sceneFilePath.textContent = `Could not load scene list location: ${error}`;
      sceneFileLocation.textContent = "";
      sceneEditorFilePath.textContent = "Could not load scene list location";
      sceneEditorFileLocation.textContent = "";
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
    } catch (error) {
      sceneEditorResult.textContent = `Could not load scene list: ${error}`;
    }
  }

  async function openSceneEditor() {
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

  async function exportSceneList() {
    exportSceneListButton.disabled = true;
    sceneListTransferResult.textContent = "Choose where to save a copy of the loaded scene list...";

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
    importSceneListButton.disabled = true;
    sceneListTransferResult.textContent = "Choose an external scene list to replace the loaded list...";

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
    sceneListTransferResult.textContent = "Replacing loaded scene list from import...";

    try {
      sceneNamesCount = await invoke("import_scene_list", {
        sourcePath: pendingSceneListImportPath,
      });
      updateReadiness();
      sceneListTransferResult.textContent = "Loaded scene list replaced from import and reloaded";
      closeSceneListImportConfirmation();

      if (!sceneEditorPanel.hidden) {
        await loadSceneEditor();
      }
    } catch (error) {
      sceneListTransferResult.textContent = `Could not replace the loaded scene list from import: ${error}`;
    } finally {
      cancelSceneListImportButton.disabled = false;
      confirmSceneListImportButton.disabled = false;
    }
  }

  async function refreshSceneListBackups(showStatus = true) {
    refreshSceneListBackupsButton.disabled = true;

    try {
      const backups = await invoke("list_scene_list_backups");
      const prompt = document.createElement("option");
      prompt.value = "";
      prompt.textContent = backups.length
        ? "Select a saved backup"
        : "No saved backups";
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
      restoreSceneListBackupButton.disabled = true;

      if (showStatus) {
        sceneListBackupResult.textContent = backups.length
          ? `${backups.length} saved backup${backups.length === 1 ? "" : "s"} available`
          : "No saved scene list backups";
      }
    } catch (error) {
      sceneListBackups.disabled = true;
      restoreSceneListBackupButton.disabled = true;
      sceneListBackupResult.textContent = `Could not load scene list backups: ${error}`;
    } finally {
      refreshSceneListBackupsButton.disabled = false;
    }
  }

  async function saveSceneListBackup() {
    saveSceneListBackupButton.disabled = true;
    sceneListBackupResult.textContent = "Saving app-managed scene list backup...";

    try {
      const backupName = await invoke("save_scene_list_backup", {
        name: sceneBackupName.value,
      });
      sceneBackupName.value = "";
      await refreshSceneListBackups(false);
      sceneListBackups.value = backupName;
      restoreSceneListBackupButton.disabled = false;
      sceneListBackupResult.textContent = `Backup saved: ${backupName}. Loaded scene list unchanged`;
    } catch (error) {
      sceneListBackupResult.textContent = `Could not save scene list backup: ${error}`;
    } finally {
      saveSceneListBackupButton.disabled = false;
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
      sceneListBackupResult.textContent = "Select a saved backup to restore to the loaded scene list";
      return;
    }

    pendingSceneListBackupName = backupName;
    restoreSceneListBackupName.textContent = `Backup to restore: ${backupName}`;
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
    sceneListBackupResult.textContent = "Replacing loaded scene list from backup...";

    try {
      sceneNamesCount = await invoke("restore_scene_list_backup", {
        backupName: pendingSceneListBackupName,
      });
      updateReadiness();
      sceneListBackupResult.textContent = "Backup restored to loaded scene list and reloaded";
      closeSceneListBackupRestoreConfirmation();

      if (!sceneEditorPanel.hidden) {
        await loadSceneEditor();
      }
    } catch (error) {
      sceneListBackupResult.textContent = `Could not restore backup to the loaded scene list: ${error}`;
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
    void saveSceneListBackup();
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
    sceneListBackupResult.textContent = "Restore cancelled; loaded scene list unchanged";
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
    midiSceneDiagnostics.textContent = "";
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
