/// Returns true when the current process is running from an MSIX package
/// (i.e. installed via Microsoft Store or `.msix`/`.msixbundle`).
///
/// The reliable indicator is that the executable path contains `\WindowsApps\`,
/// which is the protected directory where the OS unpacks MSIX packages.
/// We intentionally avoid `GetCurrentPackageFullName` WinAPI here to keep the
/// dependency surface minimal — the path check is sufficient for our purposes.
#[allow(dead_code)]
pub fn is_msix_package() -> bool {
    std::env::current_exe()
        .map(|p| {
            let s = p.to_string_lossy().to_ascii_lowercase();
            s.contains("\\windowsapps\\")
        })
        .unwrap_or(false)
}
