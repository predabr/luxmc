import re

with open('src-tauri/src/core/launcher/mod.rs', 'r') as f:
    code = f.read()

# Change launch signature
code = code.replace(
    '''pub async fn launch(
        &self,
        detail: &VersionDetail,
        username: &str,
        uuid: &str,
        access_token: &str,
        user_type: &str,
        game_dir: &PathBuf,
    ) -> AppResult<u32> {''',
    '''pub async fn launch(
        &self,
        detail: &VersionDetail,
        username: &str,
        uuid: &str,
        access_token: &str,
        user_type: &str,
        game_dir: &PathBuf,
        profile: &crate::db::models::ProfileRow,
    ) -> AppResult<u32> {'''
)

# Update build_jvm_args call
code = code.replace(
    '''let jvm_args = self.build_jvm_args(detail, &classpath, &natives_dir, game_dir);''',
    '''let jvm_args = self.build_jvm_args(detail, &classpath, &natives_dir, game_dir, profile);'''
)

# Change build_jvm_args signature
code = code.replace(
    '''fn build_jvm_args(
        &self,
        detail: &VersionDetail,
        classpath: &[PathBuf],
        natives_dir: &PathBuf,
        game_dir: &PathBuf,
    ) -> Vec<String> {''',
    '''fn build_jvm_args(
        &self,
        detail: &VersionDetail,
        classpath: &[PathBuf],
        natives_dir: &PathBuf,
        game_dir: &PathBuf,
        profile: &crate::db::models::ProfileRow,
    ) -> Vec<String> {'''
)

# Add automatic RAM to build_jvm_args end
code = code.replace(
    '''        args
    }''',
    '''
        if let Some(ref custom_args) = profile.jvm_args {
            if !custom_args.is_empty() {
                for arg in custom_args.split_whitespace() {
                    args.push(arg.to_string());
                }
            }
        }

        // Automatic RAM Optimization (if user didn't specify Xmx)
        if !args.iter().any(|a| a.starts_with("-Xmx")) {
            let mut sys = sysinfo::System::new_all();
            sys.refresh_memory();
            let total_ram = sys.total_memory(); // in bytes
            let half_ram_mb = (total_ram / 1024 / 1024 / 2) as u64;
            let target_ram = std::cmp::min(half_ram_mb, 8192); // Max 8GB by default
            if target_ram > 1024 {
                args.push(format!("-Xmx{}M", target_ram));
                args.push(format!("-Xms{}M", target_ram / 2));
            } else {
                args.push("-Xmx2G".into());
                args.push("-Xms1G".into());
            }
            
            // Performance flags
            args.push("-XX:+UseG1GC".into());
            args.push("-XX:+ParallelRefProcEnabled".into());
            args.push("-XX:MaxGCPauseMillis=200".into());
            args.push("-XX:+UnlockExperimentalVMOptions".into());
            args.push("-XX:+DisableExplicitGC".into());
            args.push("-XX:+AlwaysPreTouch".into());
            args.push("-XX:G1NewSizePercent=30".into());
            args.push("-XX:G1MaxNewSizePercent=40".into());
            args.push("-XX:G1HeapRegionSize=8M".into());
            args.push("-XX:G1ReservePercent=20".into());
            args.push("-XX:G1HeapWastePercent=5".into());
            args.push("-XX:G1MixedGCCountTarget=4".into());
            args.push("-XX:InitiatingHeapOccupancyPercent=15".into());
            args.push("-XX:G1MixedGCLiveThresholdPercent=90".into());
            args.push("-XX:G1RSetUpdatingPauseTimePercent=5".into());
            args.push("-XX:SurvivorRatio=32".into());
            args.push("-XX:+PerfDisableSharedMem".into());
            args.push("-XX:MaxTenuringThreshold=1".into());
        }

        args
    }'''
)

# Add resolution to build_game_args call
code = code.replace(
    '''let game_args =
            self.build_game_args(detail, username, uuid, access_token, user_type, game_dir);''',
    '''let game_args =
            self.build_game_args(detail, username, uuid, access_token, user_type, game_dir, profile);'''
)

code = code.replace(
    '''fn build_game_args(
        &self,
        detail: &VersionDetail,
        username: &str,
        uuid: &str,
        access_token: &str,
        user_type: &str,
        game_dir: &PathBuf,
    ) -> Vec<String> {''',
    '''fn build_game_args(
        &self,
        detail: &VersionDetail,
        username: &str,
        uuid: &str,
        access_token: &str,
        user_type: &str,
        game_dir: &PathBuf,
        profile: &crate::db::models::ProfileRow,
    ) -> Vec<String> {'''
)

code = code.replace(
    '''        let replaces = vec![''',
    '''        let width = profile.resolution_w.unwrap_or(854).to_string();
        let height = profile.resolution_h.unwrap_or(480).to_string();
        
        let mut replaces = vec![
            ("${resolution_width}", width.as_str()),
            ("${resolution_height}", height.as_str()),'''
)

with open('src-tauri/src/core/launcher/mod.rs', 'w') as f:
    f.write(code)
