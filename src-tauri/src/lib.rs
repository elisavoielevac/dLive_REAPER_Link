pub mod reaper_osc;

use std::sync::Mutex;
use tauri::Emitter;

#[derive(Default)]
struct MidiLogState {
    connection: Mutex<Option<midir::MidiInputConnection<()>>>,
}

#[tauri::command]
fn list_midi_inputs() -> Result<Vec<String>, String> {
    let midi_input =
        midir::MidiInput::new("dLive REAPER Link").map_err(|error| error.to_string())?;

    midi_input
        .ports()
        .iter()
        .map(|port| {
            midi_input
                .port_name(port)
                .map_err(|error| error.to_string())
        })
        .collect()
}

#[tauri::command]
fn start_midi_log(
    port_name: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, MidiLogState>,
) -> Result<(), String> {
    let midi_input =
        midir::MidiInput::new("dLive REAPER Link").map_err(|error| error.to_string())?;
    let port = midi_input
        .ports()
        .into_iter()
        .find(|port| midi_input.port_name(port).ok().as_deref() == Some(port_name.as_str()))
        .ok_or_else(|| format!("MIDI input not found: {port_name}"))?;

    let previous_connection = state
        .connection
        .lock()
        .map_err(|_| "MIDI log state is unavailable".to_string())?
        .take();
    drop(previous_connection);

    let connection = midi_input
        .connect(
            &port,
            "dLive REAPER Link MIDI Log",
            move |_timestamp, message, _| {
                let raw_message = message
                    .iter()
                    .map(|byte| format!("{byte:02X}"))
                    .collect::<Vec<_>>()
                    .join(" ");

                let _ = app.emit("midi-message", raw_message);

                if let [0xCB, program, ..] = message {
                    let _ = app.emit("midi-scene-recall", u16::from(*program) + 1);
                }
            },
            (),
        )
        .map_err(|error| error.to_string())?;

    let mut active_connection = state
        .connection
        .lock()
        .map_err(|_| "MIDI log state is unavailable".to_string())?;
    *active_connection = Some(connection);

    Ok(())
}

#[tauri::command]
fn test_reaper_osc_marker() -> Result<String, String> {
    reaper_osc::ReaperOscClient::default()
        .create_marker("OSC TEST")
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(MidiLogState::default())
        .invoke_handler(tauri::generate_handler![
            list_midi_inputs,
            start_midi_log,
            test_reaper_osc_marker
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
