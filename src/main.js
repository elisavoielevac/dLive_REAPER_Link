const { invoke } = window.__TAURI__.core;

window.addEventListener("DOMContentLoaded", () => {
  const testButton = document.querySelector("#test-reaper-osc");
  const testResult = document.querySelector("#test-reaper-osc-result");
  const midiButton = document.querySelector("#list-midi-inputs");
  const midiResult = document.querySelector("#midi-inputs-result");
  const midiInputSelect = document.querySelector("#midi-input-select");
  const startMidiLogButton = document.querySelector("#start-midi-log");
  const midiLogResult = document.querySelector("#midi-log-result");
  const midiLog = document.querySelector("#midi-log");
  let unlistenMidiMessages;

  testButton.addEventListener("click", async () => {
    testButton.disabled = true;
    testResult.textContent = "Testing REAPER OSC...";

    try {
      const markerNumber = await invoke("test_reaper_osc_marker");
      testResult.textContent = `Created marker ${markerNumber}`;
    } catch (error) {
      testResult.textContent = `Error: ${error}`;
    } finally {
      testButton.disabled = false;
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
      startMidiLogButton.disabled = !inputs.length;
    } catch (error) {
      midiResult.textContent = `Error: ${error}`;
    } finally {
      midiButton.disabled = false;
    }
  });

  startMidiLogButton.addEventListener("click", async () => {
    const portName = midiInputSelect.value;

    if (!portName) {
      midiLogResult.textContent = "Select a MIDI input first";
      return;
    }

    startMidiLogButton.disabled = true;
    midiLogResult.textContent = "Starting MIDI log...";
    midiLog.textContent = "";

    try {
      if (!unlistenMidiMessages) {
        unlistenMidiMessages = await window.__TAURI__.event.listen(
          "midi-message",
          (event) => {
            midiLog.textContent = `${event.payload}\n${midiLog.textContent}`;
          },
        );
      }

      await invoke("start_midi_log", { portName });
      midiLogResult.textContent = `Logging ${portName}`;
    } catch (error) {
      midiLogResult.textContent = `Error: ${error}`;
    } finally {
      startMidiLogButton.disabled = false;
    }
  });
});
