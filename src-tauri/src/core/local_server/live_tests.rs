use super::*;

#[tokio::test]
#[ignore = "Starts and stops an isolated real Minecraft server; requires explicit EULA and resource authorization"]
async fn real_server_accepts_connections_commands_and_saves_without_touching_existing_worlds() {
    assert_eq!(std::env::var("LUXMC_REAL_SERVER_TEST").as_deref(),Ok("authorized"));
    let temporary=tempfile::tempdir().unwrap();
    let manager=LocalServerManager::new(temporary.path().join("servers"),reqwest::Client::new());
    let version=std::env::var("LUXMC_REAL_SERVER_MC").unwrap_or_else(|_|"1.21.1".into());
    let kind=std::env::var("LUXMC_REAL_SERVER_LOADER").unwrap_or_else(|_|"vanilla".into());
    let config=manager.create(CreateServer{name:"Luxmc isolated connection test".into(),mc_version:version.clone(),ram_mb:2048,max_players:10,eula_accepted:true}).await.unwrap();
    let directory=manager.directory(&config.id).await.unwrap();
    assert_eq!(crate::core::managed_skin::status(&directory).await["valid"],true);
    tokio::fs::write(directory.join(crate::core::managed_skin::FILE),b"damaged fixture").await.unwrap();
    let mut settings=manager.settings(&config.id).await.unwrap();
    settings.view_distance=2;settings.simulation_distance=2;settings.generate_structures=false;settings.gamemode="creative".into();settings.difficulty="peaceful".into();
    manager.update_settings(&config.id,settings).await.unwrap();
    tokio::fs::write(directory.join("server.properties"),"spawn-chunk-radius=0\n").await.unwrap();
    if kind!="vanilla" {manager.set_loader(&config.id,loaders::ServerLoader{kind:kind.clone(),version:std::env::var("LUXMC_REAL_SERVER_LOADER_VERSION").unwrap()}).await.unwrap();}
    manager.start(config.id.clone(),10).await.unwrap();
    let result=async {
        let mut phase=String::new();
        tokio::time::timeout(Duration::from_secs(420),async {
            loop {
                let state=manager.current(&config.id).map_err(|error|error.to_string())?.snapshot(0);
                if phase!=state.phase {phase=state.phase.clone();eprintln!("LIVE {kind} {version}: {phase}");}
                if state.phase=="failed" {return Err(format!("{}",state.error.unwrap_or_default()));}
                if state.phase=="running" {break;}
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            Ok::<(),String>(())
        }).await.map_err(|error|error.to_string())??;
        assert_eq!(crate::core::managed_skin::status(&directory).await["valid"],true);
        let diagnosis=manager.connection_diagnostic(&config.id).await.map_err(|error|error.to_string())?;
        let mut diagnosis=diagnosis;
        if std::env::var("LUXMC_REAL_SERVER_P2P_RELAY").as_deref()==Ok("authorized") {
            let port=manager.running_port(&config.id).map_err(|error|error.to_string())?;
            diagnosis["p2pRelay"]=crate::network::p2p_tunnel::verify_dedicated_public_relay(port).await.map_err(|error|error.to_string())?;
        }
        assert_eq!(diagnosis["minecraftResponding"],true);assert!(diagnosis["address"].as_str().unwrap().starts_with("127.0.0.1:"));
        manager.console(&config.id,"list".into()).await.map_err(|error|error.to_string())?;
        let runtime=manager.current(&config.id).map_err(|error|error.to_string())?;
        assert!(runtime.snapshot(0).console.iter().any(|line|line.stream=="response" && line.text.contains("players online")),"RCON list response missing");
        manager.console(&config.id,"save-all flush".into()).await.map_err(|error|error.to_string())?;
        let output=manager.console_text(&config.id).await.map_err(|error|error.to_string())?;
        assert!(output["text"].as_str().unwrap().contains("players online"));
        assert!(directory.join("world/level.dat").is_file());
        let properties=tokio::fs::read_to_string(directory.join("server.properties")).await.unwrap();
        assert!(properties.contains("gamemode=creative"));assert!(properties.contains("difficulty=peaceful"));
        Ok::<_,String>(diagnosis)
    }.await;
    let stopped=manager.stop(&config.id).await;
    let state=manager.runtime.lock().unwrap().as_ref().unwrap().snapshot(0);
    if let Ok(path)=std::env::var("LUXMC_REAL_SERVER_RECEIPT") {
        let log=manager.console_text(&config.id).await.unwrap_or_default();
        std::fs::write(path,serde_json::to_vec_pretty(&serde_json::json!({"loader":kind,"minecraft":version,"testResult":result,"stopResult":stopped.as_ref().map(|_|"ok").map_err(|error|error.to_string()),"state":state,"log":log,"isolatedWorld":true})).unwrap()).unwrap();
    }
    assert!(result.is_ok(),"{result:?}; console: {:?}",state.console.iter().rev().take(25).map(|line|&line.text).collect::<Vec<_>>());
    stopped.unwrap();assert_eq!(state.phase,"stopped");assert_eq!(state.exit_code,Some(0));assert!(!state.forced_stop);assert!(state.last_saved_at.is_some());
    manager.delete_server(&config.id,&config.name,true).await.unwrap();assert!(!directory.exists());
    eprintln!("LIVE SERVER PASSED: {kind} {version}; Minecraft status, authenticated list, save, graceful stop and complete deletion");
}
