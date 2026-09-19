const { invoke } = window.__TAURI__.core;

window.addEventListener("DOMContentLoaded", () => {
  const testButton = document.querySelector("#test-reaper-osc");
  const testResult = document.querySelector("#test-reaper-osc-result");

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
});
