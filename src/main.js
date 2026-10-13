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
  const readinessSceneCount = document.querySelector("#readiness-scene-count");
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
  const manageSceneListsButton = document.querySelector("#manage-scene-lists");
  const manageSceneListsDialog = document.querySelector(
    "#manage-scene-lists-dialog",
  );
  const manageSceneListsList = document.querySelector(
    "#manage-scene-lists-list",
  );
  const manageSceneListsResult = document.querySelector(
    "#manage-scene-lists-result",
  );
  const closeManageSceneListsButton = document.querySelector(
    "#close-manage-scene-lists",
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
  const deleteSceneListConfirmation = document.querySelector(
    "#delete-scene-list-confirmation",
  );
  const deleteSceneListName = document.querySelector("#delete-scene-list-name");
  const cancelDeleteSceneListButton = document.querySelector(
    "#cancel-delete-scene-list",
  );
  const confirmDeleteSceneListButton = document.querySelector(
    "#confirm-delete-scene-list",
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
  const switchSceneListConfirmation = document.querySelector(
    "#switch-scene-list-confirmation",
  );
  const switchSceneListName = document.querySelector(
    "#switch-scene-list-name",
  );
  const switchSceneListResult = document.querySelector(
    "#switch-scene-list-result",
  );
  const cancelSwitchSceneListButton = document.querySelector(
    "#cancel-switch-scene-list",
  );
  const discardSwitchSceneListButton = document.querySelector(
    "#discard-switch-scene-list",
  );
  const saveSwitchSceneListButton = document.querySelector(
    "#save-switch-scene-list",
  );
  const sceneEditorPanel = document.querySelector("#scene-editor-panel");
  const closeSceneEditorButton = document.querySelector("#close-scene-editor");
  const discardSceneEditorConfirmation = document.querySelector(
    "#discard-scene-editor-confirmation",
  );
  const createSceneListUnsavedConfirmation = document.querySelector(
    "#create-scene-list-unsaved-confirmation",
  );
  const createSceneListUnsavedResult = document.querySelector(
    "#create-scene-list-unsaved-result",
  );
  const discardSceneEditorResult = document.querySelector(
    "#discard-scene-editor-result",
  );
  const cancelDiscardSceneEditorButton = document.querySelector(
    "#cancel-discard-scene-editor",
  );
  const discardSceneNameChangesButton = document.querySelector(
    "#discard-scene-name-changes",
  );
  const saveAndCloseSceneEditorButton = document.querySelector(
    "#save-and-close-scene-editor",
  );
  const cancelCreateSceneListUnsavedButton = document.querySelector(
    "#cancel-create-scene-list-unsaved",
  );
  const discardCreateSceneListUnsavedButton = document.querySelector(
    "#discard-create-scene-list-unsaved",
  );
  const saveCreateSceneListUnsavedButton = document.querySelector(
    "#save-create-scene-list-unsaved",
  );
  const sceneEditorFilePath = document.querySelector("#scene-editor-file-path");
  const sceneEditorHeading = document.querySelector("#scene-editor-heading");
  const sceneEditorFileLocation = document.querySelector(
    "#scene-editor-file-location",
  );
  const sceneEditor = document.querySelector("#scene-editor");
  const saveSceneNamesButton = document.querySelector("#save-scene-names");
  const sceneEditorResult = document.querySelector("#scene-editor-result");
  const midiInputSelect = document.querySelector("#midi-input-select");
  const midiListenerToggleButton = document.querySelector(
    "#toggle-midi-listener",
  );
  const midiListenerToggleState = document.querySelector(
    "#midi-listener-toggle-state",
  );
  const midiListenerStatus = document.querySelector("#midi-listener-status");
  const midiLogResult = document.querySelector("#midi-log-result");
  const midiSceneResult = document.querySelector("#midi-scene-result");
  const midiSceneNameResult = document.querySelector("#midi-scene-name-result");
  const midiSceneDiagnostics = document.querySelector(
    "#midi-scene-diagnostics",
  );
  const markerDecision = document.querySelector("#marker-decision");
  const midiLog = document.querySelector("#midi-log");
  const liveMidiRefreshButton = document.querySelector("#refresh-live-midi");
  let dLiveLinkRunning = false;
  let selectedMidiInput = "";
  let midiListeningRequested = false;
  let pendingMidiRefresh;
  let availableMidiInputs = [];
  let midiInputListKey;

  let markerMode = "recording_only";
  let savedReaperOscTarget;
  let sceneNamesCount;
  let loadedSceneListName;
  let sceneListFilePath;
  let sceneEditorDirty = false;
  let pendingSceneListImportPath;
  let pendingSceneListSwitchName;
  let pendingSceneListNameAction;
  let pendingSceneListRenameName;
  let pendingSceneListDeleteName;
  let pendingCreateSceneListName;
  let midiEventSubscriptions;
  let bridgeSnapshot;
  let bridgeSettingsLoaded = false;
  const bridgeTransport = document.querySelector("#bridge-transport");
  const bridgeHost = document.querySelector("#bridge-host");
  const bridgePort = document.querySelector("#bridge-port");
  const bridgeChannel = document.querySelector("#bridge-channel");
  const bridgeSettingsResult = document.querySelector("#bridge-settings-result");
  function updateSourceFields() {
    document.querySelector("#bridge-tcp-settings").hidden = bridgeTransport.value !== "tcp";
    document.querySelector("#bridge-midi-settings").hidden = bridgeTransport.value !== "midi";
  }
  bridgeTransport.addEventListener("change", updateSourceFields);

  function updateReadiness() {
    readinessMidiInput.textContent = bridgeSnapshot
      ? `Scene source: ${bridgeSnapshot.config.transport === "tcp" ? "MixRack TCP" : "MIDI input"} (${bridgeSnapshot.status})`
      : "Scene source: Loading";
    readinessReaperOsc.textContent = savedReaperOscTarget
      ? `REAPER OSC: Configured (${savedReaperOscTarget.host}:${savedReaperOscTarget.port})`
      : "REAPER OSC: Check settings";
    readinessSceneNames.textContent = loadedSceneListName || "No scene list loaded";
    readinessSceneCount.textContent =
      loadedSceneListName && Number.isInteger(sceneNamesCount)
        ? `${sceneNamesCount} scenes`
        : "";
  }

  function updateMidiListenerToggle() {
    const armed = dLiveLinkRunning || midiListeningRequested;
    const state = armed ? "Stop Listening" : "Start Listening";
    midiListenerToggleState.textContent = state;
    const labels = { connected: "Listening", connecting: "Connecting", waiting: "Waiting", stopped: "Stopped", error: "Error" };
    midiListenerStatus.textContent = labels[bridgeSnapshot?.status] || "Stopped";
    midiListenerStatus.title = bridgeSnapshot?.detail || "Configure a scene source in Setup.";
    midiListenerToggleButton.setAttribute("aria-label", `dLive Listener: ${state}`);
    midiListenerToggleButton.classList.toggle("is-listening", dLiveLinkRunning);
    midiListenerToggleButton.disabled = !bridgeSnapshot;
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
    sceneFilePath.textContent = loadedSceneListLabel;
    sceneEditorHeading.textContent = "Editing: " + loadedSceneListLabel;
    sceneEditorFilePath.textContent =
      loadedSceneListName ? "Loaded Scene List" : "No scene list loaded";
    sceneFileLocation.textContent = sceneListFilePath
      ? "Location: " + sceneListFilePath
      : "";
    sceneEditorFileLocation.textContent = sceneListFilePath
      ? "Location: " + sceneListFilePath
      : "";
    openSceneEditorButton.disabled = !loadedSceneListName;
    reloadSceneNamesButton.disabled = !loadedSceneListName;
    saveSceneListBackupButton.disabled = !loadedSceneListName;
    exportSceneListButton.disabled = !loadedSceneListName;
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
    discardSceneEditorResult.textContent = "";
    sceneEditorPanel.hidden = true;
    document.body.classList.remove("scene-editor-open");
    sceneEditor.replaceChildren();
    sceneEditorResult.textContent = "";
    sceneEditorDirty = false;
  }

  function closeSceneEditor() {
    if (sceneEditorDirty) {
      discardSceneEditorResult.textContent = "";
      discardSceneEditorConfirmation.hidden = false;
      saveAndCloseSceneEditorButton.focus();
      return;
    }

    closeSceneEditorNow();
  }

  function closeSceneListImportConfirmation() {
    importSceneListConfirmation.hidden = true;
    document.body.classList.remove("scene-list-import-open");
    pendingSceneListImportPath = undefined;
  }

  function openSceneListNamePrompt(action, sceneListName = "") {
    pendingSceneListNameAction = action;
    pendingSceneListRenameName = action === "rename" ? sceneListName : undefined;
    const label =
      action === "create" ? "New" : action === "save_as" ? "Save As" : "Rename";
    sceneListNamePromptHeading.textContent = label;
    confirmSceneListNamePromptButton.textContent = label;
    sceneListNamePromptInput.value = sceneListName;
    sceneListNamePromptResult.textContent = "";
    sceneListNamePrompt.hidden = false;
    document.body.classList.add("scene-list-name-prompt-open");
    sceneListNamePromptInput.focus();
    sceneListNamePromptInput.select();
  }

  function closeSceneListNamePrompt() {
    sceneListNamePrompt.hidden = true;
    document.body.classList.remove("scene-list-name-prompt-open");
    pendingSceneListNameAction = undefined;
    pendingSceneListRenameName = undefined;
    sceneListNamePromptInput.value = "";
    sceneListNamePromptResult.textContent = "";
  }

  function openCreateSceneListUnsavedConfirmation(name) {
    pendingCreateSceneListName = name;
    createSceneListUnsavedResult.textContent = "";
    createSceneListUnsavedConfirmation.hidden = false;
    discardCreateSceneListUnsavedButton.focus();
  }

  function closeCreateSceneListUnsavedConfirmation() {
    createSceneListUnsavedConfirmation.hidden = true;
    createSceneListUnsavedResult.textContent = "";
    pendingCreateSceneListName = undefined;
  }

  async function saveLoadedSceneListEdits(result) {
    const names = Array.from(sceneEditor.querySelectorAll("input"), (input) =>
      input.value,
    );

    if (names.length !== 500) {
      result.textContent = "Could not save scene list: 500 scene slots are required";
      return false;
    }

    result.textContent = "Saving scene list...";
    try {
      sceneNamesCount = await invoke("save_scene_names", { names });
      updateReadiness();
      sceneEditorDirty = false;
      result.textContent = "Scene list saved and reloaded";
      return true;
    } catch (error) {
      result.textContent = `Could not save scene list: ${error}`;
      return false;
    }
  }

  async function exportSceneList() {
    if (!loadedSceneListName) {
      sceneListTransferResult.textContent = "Load a scene list before exporting";
      return;
    }

    exportSceneListButton.disabled = true;
    sceneListTransferResult.textContent = "Choose a .drlscene export location...";

    try {
      const destinationPath = await saveFileDialog({
        title: "Export .drlscene Scene List",
        defaultPath: "dLive_REAPER_Link_scene_list.drlscene",
        filters: [
          {
            name: "dLive REAPER Link Scene Lists",
            extensions: ["drlscene"],
          },
        ],
      });

      if (!destinationPath) {
        sceneListTransferResult.textContent = "Export cancelled; loaded scene list unchanged";
        return;
      }

      await invoke("export_scene_list", { destinationPath });
      sceneListTransferResult.textContent =
        "Scene list exported as .drlscene; loaded scene list unchanged";
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
    sceneListTransferResult.textContent =
      "Choose a .drlscene or .txt list to import...";

    try {
      const sourcePath = await openFileDialog({
        title: "Import .drlscene or .txt Scene List",
        multiple: false,
        directory: false,
        filters: [
          {
            name: "Scene list files",
            extensions: ["drlscene", "txt"],
          },
        ],
      });

      if (!sourcePath || Array.isArray(sourcePath)) {
        sceneListTransferResult.textContent = "Import cancelled; loaded scene list unchanged";
        return;
      }

      pendingSceneListImportPath = sourcePath;
      importSceneListPath.textContent = sourcePath;
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

  function setSavedSceneListOptions(select, sceneLists, selectedName) {
    const prompt = document.createElement("option");
    prompt.value = "";
    prompt.textContent = sceneLists.length ? "Select a list" : "No saved lists";
    select.replaceChildren(
      prompt,
      ...sceneLists.map((sceneListName) => {
        const option = document.createElement("option");
        option.value = sceneListName;
        option.textContent = sceneListName;
        return option;
      }),
    );
    select.disabled = !sceneLists.length;
    select.value = sceneLists.includes(selectedName) ? selectedName : "";
  }

  function renderManagedSceneListRows(sceneLists) {
    if (!sceneLists.length) {
      const emptyState = document.createElement("li");
      emptyState.className = "manage-scene-lists-empty";
      emptyState.textContent = "No saved lists";
      manageSceneListsList.replaceChildren(emptyState);
      return;
    }

    manageSceneListsList.replaceChildren(
      ...sceneLists.map((sceneListName) => {
        const row = document.createElement("li");
        row.className = "manage-scene-list-row";

        const identity = document.createElement("div");
        identity.className = "manage-scene-list-identity";

        const name = document.createElement("span");
        name.className = "manage-scene-list-name";
        name.textContent = sceneListName;
        identity.append(name);

        if (sceneListName === loadedSceneListName) {
          const loaded = document.createElement("span");
          loaded.className = "manage-scene-list-loaded";
          loaded.textContent = "Loaded";
          identity.append(loaded);
        }

        const actions = document.createElement("div");
        actions.className = "manage-scene-list-actions";

        const rename = document.createElement("button");
        rename.type = "button";
        rename.textContent = "Rename";
        rename.dataset.sceneListAction = "rename";
        rename.dataset.sceneListName = sceneListName;

        const remove = document.createElement("button");
        remove.type = "button";
        remove.textContent = "Delete";
        remove.dataset.sceneListAction = "delete";
        remove.dataset.sceneListName = sceneListName;
        remove.disabled = sceneListName === loadedSceneListName;
        remove.title = remove.disabled ? "Load another list before deleting" : "";

        actions.append(rename, remove);
        row.append(identity, actions);
        return row;
      }),
    );
  }

  async function refreshSceneListBackups(showStatus = true) {
    try {
      const sceneLists = await invoke("list_saved_scene_lists");
      setSavedSceneListOptions(sceneListBackups, sceneLists, loadedSceneListName);
      renderManagedSceneListRows(sceneLists);
      manageSceneListsButton.disabled = !sceneLists.length;

      if (showStatus) {
        sceneListBackupResult.textContent = sceneLists.length
          ? `${sceneLists.length} saved list${sceneLists.length === 1 ? "" : "s"} available`
          : "No saved lists";
      }
    } catch (error) {
      sceneListBackups.disabled = true;
      manageSceneListsButton.disabled = true;
      manageSceneListsList.replaceChildren();
      sceneListBackupResult.textContent = `Could not load saved scene lists: ${error}`;
    }
  }

  function closeManageSceneLists() {
    manageSceneListsDialog.hidden = true;
    document.body.classList.remove("manage-scene-lists-open");
    manageSceneListsResult.textContent = "";
  }

  async function openManageSceneLists() {
    await refreshSceneListBackups(false);
    manageSceneListsDialog.hidden = false;
    document.body.classList.add("manage-scene-lists-open");
    manageSceneListsList.focus();
  }

  async function renameSelectedSceneList(newName) {
    if (!pendingSceneListRenameName) {
      return false;
    }

    const currentName = pendingSceneListRenameName;
    sceneListNamePromptResult.textContent = "Renaming scene list...";
    try {
      const renamedName = await invoke("rename_scene_list", {
        sceneListName: currentName,
        newName,
      });
      if (loadedSceneListName === currentName) {
        loadedSceneListName = renamedName;
        updateSceneListDisplay();
        updateReadiness();
      }
      await refreshSceneListBackups(false);
      sceneListBackupResult.textContent = `Scene list renamed: ${currentName} to ${renamedName}`;
      return true;
    } catch (error) {
      sceneListNamePromptResult.textContent = `Could not rename scene list: ${error}`;
      return false;
    }
  }

  function closeDeleteSceneListConfirmation() {
    deleteSceneListConfirmation.hidden = true;
    pendingSceneListDeleteName = undefined;
  }

  function requestSceneListDelete(sceneListName) {
    if (sceneListName === loadedSceneListName) {
      sceneListBackupResult.textContent =
        "Cannot delete the loaded scene list; load another list first";
      return;
    }

    pendingSceneListDeleteName = sceneListName;
    deleteSceneListName.textContent = `Scene list: ${sceneListName}`;
    deleteSceneListConfirmation.hidden = false;
    confirmDeleteSceneListButton.focus();
  }

  async function deleteSelectedSceneList() {
    if (!pendingSceneListDeleteName) {
      return;
    }

    const sceneListName = pendingSceneListDeleteName;
    cancelDeleteSceneListButton.disabled = true;
    confirmDeleteSceneListButton.disabled = true;
    sceneListBackupResult.textContent = "Deleting scene list...";
    try {
      await invoke("delete_scene_list", { sceneListName });
      closeDeleteSceneListConfirmation();
      await refreshSceneListBackups(false);
      sceneListBackupResult.textContent = `Scene list deleted: ${sceneListName}`;
    } catch (error) {
      sceneListBackupResult.textContent = `Could not delete scene list: ${error}`;
    } finally {
      cancelDeleteSceneListButton.disabled = false;
      confirmDeleteSceneListButton.disabled = false;
    }
  }

  async function saveSceneListBackup(name) {
    if (!loadedSceneListName) {
      sceneListBackupResult.textContent = "Load a scene list before saving as a new list";
      return false;
    }

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

  async function createPendingSceneListAfterUnsavedChoice() {
    if (!pendingCreateSceneListName) {
      return false;
    }

    const created = await createSceneList(
      pendingCreateSceneListName,
      createSceneListUnsavedResult,
    );
    if (created) {
      sceneListBackupResult.textContent =
        `Scene list created and loaded: ${loadedSceneListName}`;
      closeCreateSceneListUnsavedConfirmation();
    }
    return created;
  }

  function closeSwitchSceneListConfirmation() {
    switchSceneListConfirmation.hidden = true;
    document.body.classList.remove("scene-list-switch-open");
    switchSceneListResult.textContent = "";
    pendingSceneListSwitchName = undefined;
  }

  function restoreLoadedSceneListSelection() {
    sceneListBackups.value = loadedSceneListName || "";
  }

  async function loadSceneList(sceneListName) {
    sceneListBackups.disabled = true;
    sceneListBackupResult.textContent = "Loading scene list...";
    try {
      sceneNamesCount = await invoke("load_scene_list", { sceneListName });
      loadedSceneListName = sceneListName;
      updateSceneListDisplay();
      updateReadiness();
      await refreshSceneListBackups(false);
      sceneListBackupResult.textContent = `Scene list loaded: ${loadedSceneListName}`;

      if (!sceneEditorPanel.hidden) {
        await loadSceneEditor();
      }
      return true;
    } catch (error) {
      restoreLoadedSceneListSelection();
      sceneListBackups.disabled = sceneListBackups.options.length <= 1;
      sceneListBackupResult.textContent = `Could not load selected scene list: ${error}`;
      return false;
    }
  }

  function requestSceneListLoad() {
    const sceneListName = sceneListBackups.value;
    if (!sceneListName || sceneListName === loadedSceneListName) {
      restoreLoadedSceneListSelection();
      return;
    }

    if (!sceneEditorPanel.hidden && sceneEditorDirty) {
      pendingSceneListSwitchName = sceneListName;
      switchSceneListName.textContent = `Load: ${sceneListName}`;
      switchSceneListResult.textContent = "";
      switchSceneListConfirmation.hidden = false;
      document.body.classList.add("scene-list-switch-open");
      saveSwitchSceneListButton.focus();
      return;
    }

    void loadSceneList(sceneListName);
  }

  async function loadPendingSceneList() {
    if (!pendingSceneListSwitchName) {
      return false;
    }

    const sceneListName = pendingSceneListSwitchName;
    const loaded = await loadSceneList(sceneListName);
    if (loaded) {
      closeSwitchSceneListConfirmation();
    }
    return loaded;
  }

  // This timer only displays backend state; Rust owns listening and recovery.
  function refreshMidiInputs() {
    if (pendingMidiRefresh) return pendingMidiRefresh;
    midiButton.disabled = true;
    liveMidiRefreshButton.disabled = true;
    pendingMidiRefresh = (async () => {
      try {
        await ensureMidiEventSubscriptions();
        await refreshBridgeStatus();
        const inputs = await invoke("list_midi_inputs");
        availableMidiInputs = inputs;
        const listKey = JSON.stringify([selectedMidiInput, inputs]);
        if (midiInputListKey !== listKey) {
          const prompt = document.createElement("option");
          prompt.value = "";
          prompt.textContent = inputs.length ? "Select MIDI input" : "No MIDI inputs found";
          const names = selectedMidiInput && !inputs.includes(selectedMidiInput) ? [selectedMidiInput, ...inputs] : inputs;
          midiInputSelect.replaceChildren(prompt, ...names.map((name) => {
            const option = document.createElement("option"); option.value = name;
            option.textContent = inputs.includes(name) ? name : `${name} (unavailable)`;
            return option;
          }));
          midiInputSelect.value = selectedMidiInput;
          midiInputSelect.disabled = !names.length;
          midiInputListKey = listKey;
        }
        midiResult.textContent = inputs.length ? "" : "No MIDI inputs available";
      } catch (error) {
        midiResult.textContent = `Could not refresh input information: ${error}`;
      }
    })().finally(() => {
      pendingMidiRefresh = undefined;
      midiButton.disabled = false;
      liveMidiRefreshButton.disabled = false;
    });
    return pendingMidiRefresh;
  }

  async function refreshBridgeStatus() {
    bridgeSnapshot = await invoke("get_bridge_status");
    dLiveLinkRunning = bridgeSnapshot.status === "connected";
    midiListeningRequested = bridgeSnapshot.requested;
    if (!bridgeSettingsLoaded) {
      const config = bridgeSnapshot.config;
      selectedMidiInput = config.midi_input;
      bridgeTransport.value = config.transport;
      bridgeHost.value = config.host;
      bridgePort.value = config.port;
      bridgeChannel.value = config.channel;
      bridgeSettingsLoaded = true;
      updateSourceFields();
    }
    midiLogResult.textContent = bridgeSnapshot.detail || "dLive Listener stopped";
    updateMidiListenerToggle();
    updateReadiness();
  }

  const saveBridgeButton = document.querySelector("#save-bridge-settings");
  saveBridgeButton.addEventListener("click", async () => {
    saveBridgeButton.disabled = true;
    try {
      await invoke("save_bridge_settings", { config: {
        transport: bridgeTransport.value, host: bridgeHost.value.trim(),
        port: Number(bridgePort.value), channel: Number(bridgeChannel.value),
        midi_input: selectedMidiInput,
      } });
      bridgeSettingsResult.textContent = "Scene source saved";
      clearMidiSceneStatus();
      await refreshBridgeStatus();
    } catch (error) { bridgeSettingsResult.textContent = `Could not save scene source: ${error}`; }
    finally { saveBridgeButton.disabled = false; }
  });
  document.querySelector("#reconnect-bridge").addEventListener("click", async () => {
    try { await invoke("reconnect_bridge"); await refreshBridgeStatus(); }
    catch (error) { bridgeSettingsResult.textContent = `Could not reconnect: ${error}`; }
  });

  void refreshMidiInputs();
  window.setInterval(() => { void refreshMidiInputs(); }, 2000);

  revealSceneFileButton.addEventListener("click", async () => {
    revealSceneFileButton.disabled = true;
    sceneFileRevealResult.textContent = "Opening scene lists folder...";

    try {
      await invoke("reveal_scene_file");
      sceneFileRevealResult.textContent = "Scene lists folder opened";
    } catch (error) {
      sceneFileRevealResult.textContent =
        `Could not open scene lists folder: ${error}`;
    } finally {
      revealSceneFileButton.disabled = false;
    }
  });

  reloadSceneNamesButton.addEventListener("click", async () => {
    if (!loadedSceneListName) {
      sceneFileReloadResult.textContent = "Load a scene list before reloading";
      return;
    }

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
    saveSceneNamesButton.disabled = true;
    closeSceneEditorButton.disabled = true;
    try {
      await saveLoadedSceneListEdits(sceneEditorResult);
    } finally {
      saveSceneNamesButton.disabled = false;
      closeSceneEditorButton.disabled = false;
    }
  });

  createSceneListButton.addEventListener("click", async () => {
    openSceneListNamePrompt("create");
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
    if (!loadedSceneListName) {
      sceneListBackupResult.textContent = "Load a scene list before saving as a new list";
      return;
    }
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
        if (!sceneEditorPanel.hidden && sceneEditorDirty) {
          closeSceneListNamePrompt();
          openCreateSceneListUnsavedConfirmation(name);
          return;
        } else {
          completed = await createSceneList(name, sceneListNamePromptResult);
          if (completed) {
            sceneListBackupResult.textContent =
              `Scene list created and loaded: ${loadedSceneListName}`;
          }
        }
      } else if (pendingSceneListNameAction === "save_as") {
        completed = await saveSceneListBackup(name);
      } else if (pendingSceneListNameAction === "rename") {
        completed = await renameSelectedSceneList(name);
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

  sceneListBackups.addEventListener("change", () => {
    requestSceneListLoad();
  });

  manageSceneListsButton.addEventListener("click", () => {
    void openManageSceneLists();
  });

  closeManageSceneListsButton.addEventListener("click", closeManageSceneLists);

  manageSceneListsList.addEventListener("click", (event) => {
    const button = event.target.closest("button[data-scene-list-action]");
    if (!button || !manageSceneListsList.contains(button)) {
      return;
    }

    const sceneListName = button.dataset.sceneListName;
    if (!sceneListName) {
      return;
    }

    if (button.dataset.sceneListAction === "rename") {
      openSceneListNamePrompt("rename", sceneListName);
    } else if (button.dataset.sceneListAction === "delete") {
      requestSceneListDelete(sceneListName);
    }
  });

  cancelSwitchSceneListButton.addEventListener("click", () => {
    closeSwitchSceneListConfirmation();
    restoreLoadedSceneListSelection();
    sceneListBackupResult.textContent = "Load cancelled; loaded scene list unchanged";
  });

  discardSwitchSceneListButton.addEventListener("click", async () => {
    discardSwitchSceneListButton.disabled = true;
    saveSwitchSceneListButton.disabled = true;
    cancelSwitchSceneListButton.disabled = true;
    try {
      await loadPendingSceneList();
    } finally {
      discardSwitchSceneListButton.disabled = false;
      saveSwitchSceneListButton.disabled = false;
      cancelSwitchSceneListButton.disabled = false;
    }
  });

  saveSwitchSceneListButton.addEventListener("click", async () => {
    discardSwitchSceneListButton.disabled = true;
    saveSwitchSceneListButton.disabled = true;
    cancelSwitchSceneListButton.disabled = true;
    try {
      const saved = await saveLoadedSceneListEdits(switchSceneListResult);
      if (saved) {
        await loadPendingSceneList();
      }
    } finally {
      discardSwitchSceneListButton.disabled = false;
      saveSwitchSceneListButton.disabled = false;
      cancelSwitchSceneListButton.disabled = false;
    }
  });

  cancelDeleteSceneListButton.addEventListener("click", () => {
    closeDeleteSceneListConfirmation();
    sceneListBackupResult.textContent = "Delete cancelled";
  });

  confirmDeleteSceneListButton.addEventListener("click", () => {
    void deleteSelectedSceneList();
  });

  closeSceneEditorButton.addEventListener("click", closeSceneEditor);

  cancelDiscardSceneEditorButton.addEventListener("click", () => {
    discardSceneEditorConfirmation.hidden = true;
    closeSceneEditorButton.focus();
  });

  discardSceneNameChangesButton.addEventListener("click", closeSceneEditorNow);

  saveAndCloseSceneEditorButton.addEventListener("click", async () => {
    cancelDiscardSceneEditorButton.disabled = true;
    discardSceneNameChangesButton.disabled = true;
    saveAndCloseSceneEditorButton.disabled = true;
    try {
      const saved = await saveLoadedSceneListEdits(discardSceneEditorResult);
      if (saved) {
        closeSceneEditorNow();
      }
    } finally {
      cancelDiscardSceneEditorButton.disabled = false;
      discardSceneNameChangesButton.disabled = false;
      saveAndCloseSceneEditorButton.disabled = false;
    }
  });

  cancelCreateSceneListUnsavedButton.addEventListener("click", () => {
    closeCreateSceneListUnsavedConfirmation();
    sceneListBackupResult.textContent = "Create new cancelled";
  });

  discardCreateSceneListUnsavedButton.addEventListener("click", async () => {
    discardCreateSceneListUnsavedButton.disabled = true;
    saveCreateSceneListUnsavedButton.disabled = true;
    cancelCreateSceneListUnsavedButton.disabled = true;
    try {
      await createPendingSceneListAfterUnsavedChoice();
    } finally {
      discardCreateSceneListUnsavedButton.disabled = false;
      saveCreateSceneListUnsavedButton.disabled = false;
      cancelCreateSceneListUnsavedButton.disabled = false;
    }
  });

  saveCreateSceneListUnsavedButton.addEventListener("click", async () => {
    discardCreateSceneListUnsavedButton.disabled = true;
    saveCreateSceneListUnsavedButton.disabled = true;
    cancelCreateSceneListUnsavedButton.disabled = true;
    try {
      const saved = await saveLoadedSceneListEdits(sceneEditorResult);
      if (saved) {
        await createPendingSceneListAfterUnsavedChoice();
      }
    } finally {
      discardCreateSceneListUnsavedButton.disabled = false;
      saveCreateSceneListUnsavedButton.disabled = false;
      cancelCreateSceneListUnsavedButton.disabled = false;
    }
  });

  sceneEditor.addEventListener("input", () => {
    sceneEditorDirty = true;
  });

  midiButton.addEventListener("click", () => {
    void refreshMidiInputs();
  });

  liveMidiRefreshButton.addEventListener("click", () => {
    void refreshMidiInputs();
  });

  midiInputSelect.addEventListener("change", () => {
    selectedMidiInput = midiInputSelect.value;
    bridgeSettingsResult.textContent = "Choose Save source to apply this selection.";
  });

  function clearMidiSceneStatus() {
    midiSceneResult.textContent = "";
    midiSceneNameResult.textContent = "";
    midiSceneDiagnostics.textContent = "";
    markerDecision.textContent = "";
  }

  async function ensureMidiEventSubscriptions() {
    if (midiEventSubscriptions) return;
    const subscriptions = [];

    try {
      subscriptions.push(await window.__TAURI__.event.listen("open-preferences", () => showAppView(2)));
      subscriptions.push(await window.__TAURI__.event.listen("bridge-status", () => { void refreshBridgeStatus().catch((error) => { midiLogResult.textContent = `Could not read connection status: ${error}`; }); }));
      subscriptions.push(await window.__TAURI__.event.listen("midi-message", (event) => {
        midiLog.textContent = `${event.payload}\n${midiLog.textContent}`.split("\n").slice(0, 200).join("\n");
      }));
      subscriptions.push(await window.__TAURI__.event.listen("midi-scene-recall", (event) => {
        const [sceneNumber, sceneName, program, channel, bank, bankLsb] = event.payload;
        midiSceneResult.textContent = `Detected scene ${sceneNumber}`;
        midiSceneNameResult.textContent = `Scene name: ${sceneName}`;
        midiSceneDiagnostics.textContent = `Scene recall: raw Program Change ${program}, MIDI channel ${channel}, bank ${bank} (LSB ${bankLsb}), decoded scene ${sceneNumber}, ${sceneName}`;
      }));
      subscriptions.push(await window.__TAURI__.event.listen("marker-decision", (event) => {
        markerDecision.textContent = event.payload;
      }));
      midiEventSubscriptions = subscriptions;
    } catch (error) {
      subscriptions.forEach((unlisten) => unlisten());
      throw error;
    }
  }

  midiListenerToggleButton.addEventListener("click", async () => {
    midiListenerToggleButton.disabled = true;
    try {
      await invoke("set_bridge_listening", { requested: !midiListeningRequested });
      clearMidiSceneStatus();
      await refreshBridgeStatus();
    } catch (error) { midiLogResult.textContent = `Could not change listening: ${error}`; }
    finally { midiListenerToggleButton.disabled = !bridgeSnapshot; }
  });
});
