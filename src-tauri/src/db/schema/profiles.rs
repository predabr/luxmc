use crate::db::models::ProfileRow;
use crate::db::Db;
use crate::error::AppError;
use crate::error::AppResult;

pub async fn list(db: &Db) -> AppResult<Vec<ProfileRow>> {
    let rows = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles ORDER BY created_at ASC")
        .fetch_all(db.pool())
        .await?;
    Ok(rows)
}

pub async fn upsert(db: &Db, p: &ProfileRow) -> AppResult<()> {
    sqlx::query(
        r#"
		INSERT INTO profiles
			(id, name, icon, mc_version, loader, loader_version, java_path, jvm_args,
			 resolution_w, resolution_h, fullscreen, game_dir, created_at, updated_at,
			 favorite, notes, last_played, launch_count, mod_count, disk_usage, ram_mb, instance_group)
		VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)
		ON CONFLICT(id) DO UPDATE SET
			name=excluded.name, icon=excluded.icon, mc_version=excluded.mc_version,
			loader=excluded.loader, loader_version=excluded.loader_version,
			java_path=excluded.java_path, jvm_args=excluded.jvm_args,
			resolution_w=excluded.resolution_w, resolution_h=excluded.resolution_h,
			fullscreen=excluded.fullscreen, game_dir=excluded.game_dir,
			updated_at=excluded.updated_at,
			favorite=excluded.favorite, notes=excluded.notes,
			last_played=excluded.last_played, launch_count=excluded.launch_count,
			mod_count=excluded.mod_count, disk_usage=excluded.disk_usage,
			ram_mb=excluded.ram_mb, instance_group=excluded.instance_group
		"#,
    )
    .bind(&p.id)
    .bind(&p.name)
    .bind(&p.icon)
    .bind(&p.mc_version)
    .bind(&p.loader)
    .bind(&p.loader_version)
    .bind(&p.java_path)
    .bind(&p.jvm_args)
    .bind(p.resolution_w)
    .bind(p.resolution_h)
    .bind(p.fullscreen)
    .bind(&p.game_dir)
    .bind(p.created_at)
    .bind(p.updated_at)
    .bind(p.favorite)
    .bind(&p.notes)
    .bind(p.last_played)
    .bind(p.launch_count)
    .bind(p.mod_count)
    .bind(p.disk_usage)
    .bind(p.ram_mb)
    .bind(&p.instance_group)
    .execute(db.pool())
    .await
    .map_err(AppError::from)?;
    Ok(())
}

pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM profiles WHERE id = ?")
        .bind(id)
        .execute(db.pool())
        .await?;
    Ok(())
}
