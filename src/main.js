const { invoke } = window.__TAURI__.core;

window.addEventListener("DOMContentLoaded", () => {
  const testButton = document.querySelector("#test-reaper-osc");
  const testResult = document.querySelector("#test-reaper-osc-result");
  const midiButton = document.querySelector("#list-midi-inputs");
  const midiResult = document.querySelector("#midi-inputs-result");

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
    } catch (error) {
      midiResult.textContent = `Error: ${error}`;
    } finally {
      midiButton.disabled = false;
    }
  });
});
