/// build.rs — Embeds Windows version info & icon into the release .exe.
/// Only active on Windows; skipped on other platforms.
fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set("ProductName", "TextMacro");
        res.set("FileDescription", "TextMacro – Text Macro Automation Tool");
        res.set("LegalCopyright", "© 2025 TextMacro");
        res.set("ProductVersion", "0.1.0");
        res.set("FileVersion", "0.1.0");

        // Use the .ico in the assets folder if it exists
        let ico_path = std::path::Path::new("assets/logo.ico");
        if ico_path.exists() {
            res.set_icon("assets/logo.ico");
        }

        if let Err(e) = res.compile() {
            // Non-fatal: just warn and continue building without resources
            eprintln!("cargo:warning=winresource compile failed: {e}");
        }
    }
}
