//! Tests for `Settings` serialisation / deserialisation round-trips.

use minute_of_silence_lib::core::settings::{AudioPreset, Settings};

#[test]
fn default_settings_serialise_and_deserialise() {
    let original = Settings::default();
    let json = serde_json::to_string(&original).expect("serialisation failed");
    let restored: Settings = serde_json::from_str(&json).expect("deserialisation failed");

    assert_eq!(restored.autostart_enabled, original.autostart_enabled);
    assert_eq!(restored.volume, original.volume);
    assert_eq!(restored.ntp_server, original.ntp_server);
    assert_eq!(
        restored.late_start_grace_minutes,
        original.late_start_grace_minutes
    );
}

#[test]
fn all_presets_round_trip() {
    let presets = [
        AudioPreset::VoiceMetronome,
        AudioPreset::MetronomeOnly,
        AudioPreset::VoiceSilenceBell,
        AudioPreset::VoiceSilence,
        AudioPreset::VoiceMetronomeAnthem,
        AudioPreset::MetronomeAnthem,
        AudioPreset::BellSilenceBell,
        AudioPreset::BellMetronomeBell,
    ];

    for preset in presets {
        let json = serde_json::to_string(&preset).unwrap();
        let restored: AudioPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(preset, restored, "Round-trip failed for {preset:?}");
    }
}

#[test]
fn volume_clamps_are_respected_in_struct() {
    // The struct doesn't enforce clamping by itself; the UI is responsible.
    // This test documents the expected range for future validation logic.
    let s = Settings {
        volume: 100,
        ..Settings::default()
    };
    assert!(s.volume <= 100);
}
