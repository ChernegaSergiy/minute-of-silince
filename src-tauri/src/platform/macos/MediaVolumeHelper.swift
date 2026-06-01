import Foundation
import AppKit
import CoreAudio

// Helper to run AppleScript in-process
private func runAppleScript(_ source: String) -> String? {
    guard let script = NSAppleScript(source: source) else { return nil }
    var error: NSDictionary?
    let result = script.executeAndReturnError(&error)
    if error != nil {
        return nil
    }
    return result.stringValue
}

// Helper to get default output audio device
private func getDefaultOutputDevice() -> AudioObjectID? {
    var deviceID = AudioObjectID(kAudioObjectUnknown)
    var size = UInt32(MemoryLayout.size(ofValue: deviceID))
    var address = AudioObjectPropertyAddress(
        mSelector: kAudioHardwarePropertyDefaultOutputDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: AudioObjectPropertyElement(kAudioObjectPropertyElementMain)
    )
    let status = AudioObjectGetPropertyData(
        AudioObjectID(kAudioSystemObject),
        &address,
        0,
        nil,
        &size,
        &deviceID
    )
    return status == noErr ? deviceID : nil
}

@_cdecl("macos_detect_system_theme")
public func macosDetectSystemTheme() -> Bool {
    if #available(macOS 10.14, *) {
        if let style = UserDefaults.standard.string(forKey: "AppleInterfaceStyle") {
            return style.lowercased().contains("dark")
        }
        let appearance = NSAppearance.current
        return appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
    }
    return false
}

@_cdecl("macos_get_volume")
public func macosGetVolume() -> UInt8 {
    guard let deviceID = getDefaultOutputDevice() else { return 0 }
    var volume = Float32(0.0)
    var size = UInt32(MemoryLayout.size(ofValue: volume))
    var address = AudioObjectPropertyAddress(
        mSelector: kAudioDevicePropertyVolumeScalar,
        mScope: kAudioDevicePropertyScopeOutput,
        mElement: AudioObjectPropertyElement(kAudioObjectPropertyElementMain)
    )
    
    var status = AudioObjectGetPropertyData(deviceID, &address, 0, nil, &size, &volume)
    if status != noErr {
        // Fallback to ElementMaster (0) if Main fails or is not supported
        address.mElement = 0
        status = AudioObjectGetPropertyData(deviceID, &address, 0, nil, &size, &volume)
    }
    
    return status == noErr ? UInt8(volume * 100.0) : 0
}

@_cdecl("macos_set_volume")
public func macosSetVolume(level: UInt8) -> Bool {
    guard let deviceID = getDefaultOutputDevice() else { return false }
    var volume = Float32(level) / 100.0
    let size = UInt32(MemoryLayout.size(ofValue: volume))
    var address = AudioObjectPropertyAddress(
        mSelector: kAudioDevicePropertyVolumeScalar,
        mScope: kAudioDevicePropertyScopeOutput,
        mElement: AudioObjectPropertyElement(kAudioObjectPropertyElementMain)
    )
    
    var status = AudioObjectSetPropertyData(deviceID, &address, 0, nil, size, &volume)
    if status != noErr {
        address.mElement = 0
        status = AudioObjectSetPropertyData(deviceID, &address, 0, nil, size, &volume)
    }
    
    return status == noErr
}

@_cdecl("macos_is_muted")
public func macosIsMuted() -> Bool {
    guard let deviceID = getDefaultOutputDevice() else { return false }
    var mute = UInt32(0)
    var size = UInt32(MemoryLayout.size(ofValue: mute))
    var address = AudioObjectPropertyAddress(
        mSelector: kAudioDevicePropertyMute,
        mScope: kAudioDevicePropertyScopeOutput,
        mElement: AudioObjectPropertyElement(kAudioObjectPropertyElementMain)
    )
    
    var status = AudioObjectGetPropertyData(deviceID, &address, 0, nil, &size, &mute)
    if status != noErr {
        address.mElement = 0
        status = AudioObjectGetPropertyData(deviceID, &address, 0, nil, &size, &mute)
    }
    
    return status == noErr && mute != 0
}

@_cdecl("macos_set_mute")
public func macosSetMute(mute: Bool) -> Bool {
    guard let deviceID = getDefaultOutputDevice() else { return false }
    var muteVal = UInt32(mute ? 1 : 0)
    let size = UInt32(MemoryLayout.size(ofValue: muteVal))
    var address = AudioObjectPropertyAddress(
        mSelector: kAudioDevicePropertyMute,
        mScope: kAudioDevicePropertyScopeOutput,
        mElement: AudioObjectPropertyElement(kAudioObjectPropertyElementMain)
    )
    
    var status = AudioObjectSetPropertyData(deviceID, &address, 0, nil, size, &muteVal)
    if status != noErr {
        address.mElement = 0
        status = AudioObjectSetPropertyData(deviceID, &address, 0, nil, size, &muteVal)
    }
    
    return status == noErr
}

@_cdecl("macos_pause_all")
public func macosPauseAll() -> UnsafeMutablePointer<Int8>? {
    let runningApps = NSWorkspace.shared.runningApplications
    var pausedBundleIDs = [String]()
    
    for app in runningApps {
        guard let bundleID = app.bundleIdentifier,
              let name = app.localizedName else { continue }
        
        let script = """
        tell application "\(name)"
            try
                if player state is playing then
                    pause
                    return "paused"
                end if
            end try
            return "not_playing"
        end tell
        """
        
        if let result = runAppleScript(script), result == "paused" {
            pausedBundleIDs.append(bundleID)
        }
    }
    
    let joined = pausedBundleIDs.joined(separator: ",")
    return strdup(joined)
}

@_cdecl("macos_resume_players")
public func macosResumePlayers(bundleIDsCsv: UnsafePointer<Int8>) {
    let csv = String(cString: bundleIDsCsv)
    let bundleIDs = csv.split(separator: ",").map(String.init)
    
    for bundleID in bundleIDs {
        let apps = NSWorkspace.shared.runningApplications
        if let app = apps.first(where: { $0.bundleIdentifier == bundleID }),
           let name = app.localizedName {
            let script = """
            tell application "\(name)"
                try
                    play
                end try
            end tell
            """
            _ = runAppleScript(script)
        }
    }
}

@_cdecl("macos_free_string")
public func macosFreeString(_ ptr: UnsafeMutablePointer<Int8>) {
    free(ptr)
}
