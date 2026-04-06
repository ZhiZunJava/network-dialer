fn main() {
    // 仅在 release 打包时嵌入管理员权限 manifest
    // 开发模式(debug)不要求管理员权限，避免 `tauri dev` 报 OS error 740
    #[cfg(target_os = "windows")]
    {
        let mut windows = tauri_build::WindowsAttributes::new();
        if !cfg!(debug_assertions) {
            windows = windows.app_manifest(include_str!("tauri.windows.manifest.xml"));
        }
        let attrs = tauri_build::Attributes::new().windows_attributes(windows);
        tauri_build::try_build(attrs).expect("failed to run tauri-build");
    }

    #[cfg(not(target_os = "windows"))]
    {
        tauri_build::build();
    }
}
