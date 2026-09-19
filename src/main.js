const { invoke } = window.__TAURI__.core;

window.addEventListener("DOMContentLoaded", () => {
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

  async function loadSceneFilePath() {
    try {
      const path = await invoke("get_scene_file_path");
      sceneFilePath.textContent = `Scene file: ${path}`;
    } catch (error) {
      sceneFilePath.textContent = `Scene file error: ${error}`;
    }
  }

  void loadSceneFilePath();

  revealSceneFileButton.addEventListener("click", async () => {
    revealSceneFileButton.disabled = true;
    sceneFileRevealResult.textContent = "Revealing scene file...";

    try {
      await invoke("reveal_scene_file");
      sceneFileRevealResult.textContent = "Scene file revealed";
    } catch (error) {
      sceneFileRevealResult.textContent = `Error: ${error}`;
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
      sceneFileReloadResult.textContent = `Error: ${error}`;
    } finally {
      reloadSceneNamesButton.disabled = false;
    }
  });

  midiButton.addEventListener("click", async () => {
    midiButton.disabled = true;
    midiResult.textContent = "Listing MIDI inputs...";

    try {
      const inputs = await invoke("list_midi_inputs");
      midiResult.textContent = inputs.length
        ? inputs.join(", ")
        : "No MIDI inputs found";
      midiInputSelect.replaceChildren(
        ...inputs.map((name) => {
          const option = document.createElement("option");
          option.value = name;
          option.textContent = name;
          return option;
        }),
      );
      midiInputSelect.disabled = !inputs.length;
      startMidiLogButton.disabled = dLiveLinkRunning || !inputs.length;
    } catch (error) {
      midiResult.textContent = `Error: ${error}`;
    } finally {
      midiButton.disabled = false;
    }
  });

  startMidiLogButton.addEventListener("click", async () => {
    if (dLiveLinkRunning) {
      midiLogResult.textContent = "dLive Link is already running";
      return;
    }

    const portName = midiInputSelect.value;

    if (!portName) {
      midiLogResult.textContent = "Select a MIDI input first";
      return;
    }

    startMidiLogButton.disabled = true;
    midiLogResult.textContent = "Starting MIDI log...";
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
      midiLogResult.textContent = `Logging ${portName}`;
      stopMidiLogButton.disabled = false;
    } catch (error) {
      midiLogResult.textContent = `Error: ${error}`;
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
      midiLogResult.textContent = `Error: ${error}`;
    } finally {
      stopMidiLogButton.disabled = !dLiveLinkRunning;
    }
  });
});
