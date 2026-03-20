use auto_launch::AutoLaunchBuilder;

pub fn set_run_on_startup(enable: bool) {
    let app_name = "TextMacro";
    let app_path = match std::env::current_exe() {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(e) => {
            log::error!(target: "engine", "Failed to get current executable path: {}", e);
            return;
        }
    };

    let auto = AutoLaunchBuilder::new()
        .set_app_name(app_name)
        .set_app_path(&app_path)
        .set_use_launch_agent(true)
        .build()
        .unwrap();

    if enable {
        if let Err(e) = auto.enable() {
            log::error!(target: "engine", "Failed to enable run on startup: {}", e);
        } else {
            log::info!(target: "engine", "Successfully enabled run on startup");
        }
    } else {
        if let Err(e) = auto.disable() {
            log::error!(target: "engine", "Failed to disable run on startup: {}", e);
        } else {
            log::info!(target: "engine", "Successfully disabled run on startup");
        }
    }
}
