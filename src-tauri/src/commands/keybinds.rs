use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeybindEntry {
    pub id: String,
    pub label: String,
    pub category: String,
    pub raw_key: String,
    pub display_key: String,
    pub is_conflict: bool,
    pub conflicting_with: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeybindListResult {
    pub keybinds: Vec<KeybindEntry>,
    pub total_conflicts: usize,
}

fn format_key_display(raw: &str) -> String {
    let clean = raw.strip_prefix("key.keyboard.").unwrap_or(raw);
    let clean = clean.strip_prefix("key.mouse.").unwrap_or(clean);

    match clean {
        "space" => "Espaço".into(),
        "left.shift" => "L-Shift".into(),
        "right.shift" => "R-Shift".into(),
        "left.control" => "L-Ctrl".into(),
        "right.control" => "R-Ctrl".into(),
        "left.alt" => "L-Alt".into(),
        "right.alt" => "R-Alt".into(),
        "tab" => "Tab".into(),
        "enter" => "Enter".into(),
        "escape" => "Esc".into(),
        "backspace" => "Backspace".into(),
        "slash" => "/".into(),
        "unknown" => "Nenhuma".into(),
        "left" => "Botão Esquerdo".into(),
        "right" => "Botão Direito".into(),
        "middle" => "Botão Meio".into(),
        other => other.to_uppercase(),
    }
}

fn categorize_key(id: &str) -> (&'static str, &'static str) {
    match id {
        "key_key.forward" => ("Movimentação", "Andar para Frente"),
        "key_key.back" => ("Movimentação", "Andar para Trás"),
        "key_key.left" => ("Movimentação", "Esquerda"),
        "key_key.right" => ("Movimentação", "Direita"),
        "key_key.jump" => ("Movimentação", "Pular"),
        "key_key.sneak" => ("Movimentação", "Agachar"),
        "key_key.sprint" => ("Movimentação", "Correr"),

        "key_key.attack" => ("Combate", "Atacar / Destruir"),
        "key_key.use" => ("Combate", "Usar Item / Colocar"),
        "key_key.pickItem" => ("Combate", "Copiar Bloco"),
        "key_key.drop" => ("Combate", "Largar Item"),
        "key_key.swapOffhand" => ("Combate", "Trocar Item de Mão"),

        "key_key.inventory" => ("Interface", "Abrir Inventário"),
        "key_key.chat" => ("Interface", "Abrir Chat"),
        "key_key.command" => ("Interface", "Abrir Linha de Comando"),
        "key_key.playerlist" => ("Interface", "Lista de Jogadores (Tab)"),
        "key_key.screenshot" => ("Interface", "Tirar Captura de Tela (F2)"),
        "key_key.togglePerspective" => ("Interface", "Mudar Visão da Câmera (F5)"),
        "key_key.smoothCamera" => ("Interface", "Câmera Cinemática"),
        "key_key.fullscreen" => ("Interface", "Tela Cheia (F11)"),
        "key_key.spectatorOutlines" => ("Interface", "Destacar Jogadores"),
        "key_key.advancements" => ("Interface", "Progresso / Avanços (L)"),

        _ => {
            let label = id.strip_prefix("key_key.").unwrap_or(id).replace('.', " ");
            ("Mods & Diversos", Box::leak(label.into_boxed_str()))
        }
    }
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn keybinds_list(profileId: String) -> AppResult<KeybindListResult> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let options_path = PathBuf::from(&row.game_dir).join("options.txt");
    if !options_path.exists() {
        return Ok(KeybindListResult {
            keybinds: Vec::new(),
            total_conflicts: 0,
        });
    }

    let content = tokio::fs::read_to_string(&options_path).await.unwrap_or_default();
    let mut raw_map: HashMap<String, String> = HashMap::new();

    for line in content.lines() {
        if let Some((k, v)) = line.split_once(':') {
            if k.starts_with("key_key.") {
                raw_map.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }

    let mut bound_count: HashMap<String, Vec<String>> = HashMap::new();
    for (id, val) in &raw_map {
        if val != "key.keyboard.unknown" && !val.is_empty() {
            bound_count.entry(val.clone()).or_default().push(id.clone());
        }
    }

    let mut keybinds = Vec::new();
    let mut total_conflicts = 0usize;

    for (id, val) in raw_map {
        let (category, label) = categorize_key(&id);
        let same_key_list = bound_count.get(&val);
        let is_conflict = same_key_list.map_or(false, |list| list.len() > 1);
        let conflicting_with = if is_conflict {
            total_conflicts += 1;
            same_key_list.unwrap().iter().filter(|&item| item != &id).cloned().collect()
        } else {
            Vec::new()
        };

        keybinds.push(KeybindEntry {
            id,
            label: label.to_string(),
            category: category.to_string(),
            raw_key: val.clone(),
            display_key: format_key_display(&val),
            is_conflict,
            conflicting_with,
        });
    }

    keybinds.sort_by(|a, b| {
        if a.is_conflict != b.is_conflict {
            b.is_conflict.cmp(&a.is_conflict)
        } else if a.category != b.category {
            a.category.cmp(&b.category)
        } else {
            a.label.cmp(&b.label)
        }
    });

    Ok(KeybindListResult {
        keybinds,
        total_conflicts: total_conflicts / 2,
    })
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn keybinds_update(
    profileId: String,
    updates: HashMap<String, String>,
) -> AppResult<bool> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let options_path = PathBuf::from(&row.game_dir).join("options.txt");
    if !options_path.exists() {
        return Ok(false);
    }

    let content = tokio::fs::read_to_string(&options_path).await.unwrap_or_default();
    let mut new_lines = Vec::new();

    for line in content.lines() {
        if let Some((k, _)) = line.split_once(':') {
            if let Some(new_val) = updates.get(k.trim()) {
                new_lines.push(format!("{}:{}", k.trim(), new_val.trim()));
                continue;
            }
        }
        new_lines.push(line.to_string());
    }

    let result_str = new_lines.join("\n");
    tokio::fs::write(&options_path, result_str.as_bytes()).await?;

    Ok(true)
}
