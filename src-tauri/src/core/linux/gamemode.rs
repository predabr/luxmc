#[cfg(target_os = "linux")]
use zbus::Connection;

pub async fn request_gamemode_for_pid(pid: u32) -> bool {
    #[cfg(target_os = "linux")]
    {
        if let Ok(connection) = Connection::session().await {
            let proxy_res = zbus::Proxy::new(
                &connection,
                "com.feralinteractive.GameMode",
                "/com/feralinteractive/GameMode",
                "com.feralinteractive.GameMode",
            )
            .await;

            if let Ok(proxy) = proxy_res {
                let call_res: Result<i32, _> = proxy.call("RegisterGame", &(pid as i32)).await;
                if let Ok(code) = call_res {
                    return code == 0;
                }
            }
        }
        false
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        false
    }
}

pub async fn release_gamemode_for_pid(pid: u32) -> bool {
    #[cfg(target_os = "linux")]
    {
        if let Ok(connection) = Connection::session().await {
            let proxy_res = zbus::Proxy::new(
                &connection,
                "com.feralinteractive.GameMode",
                "/com/feralinteractive/GameMode",
                "com.feralinteractive.GameMode",
            )
            .await;

            if let Ok(proxy) = proxy_res {
                let call_res: Result<i32, _> = proxy.call("UnregisterGame", &(pid as i32)).await;
                if let Ok(code) = call_res {
                    return code == 0;
                }
            }
        }
        false
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        false
    }
}
