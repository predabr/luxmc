use std::io::Write;
use zip::write::FileOptions;
use zip::ZipWriter;

#[test]
fn test_mrpack_manifest_parsing_with_minecraft_string() {
	let manifest_json = r#"{
		"formatVersion": 1,
		"game": "minecraft",
		"versionId": "5.13.1",
		"name": "Fabulously Optimized",
		"summary": "Simple Minecraft optimization modpack",
		"files": [
			{
				"path": "mods/sodium-fabric-0.6.0.jar",
				"hashes": {
					"sha1": "abcdef1234567890",
					"sha512": "fedcba0987654321"
				},
				"env": {
					"client": "required",
					"server": "unsupported"
				},
				"downloads": [
					"https://cdn.modrinth.com/data/AANobbMI/versions/123/sodium-fabric.jar"
				],
				"fileSize": 123456
			}
		],
		"dependencies": {
			"minecraft": "1.21.1",
			"fabric-loader": "0.16.5"
		}
	}"#;

	let mut buf = Vec::new();
	{
		let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buf));
		let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
		zip.start_file("modrinth.index.json", options).unwrap();
		zip.write_all(manifest_json.as_bytes()).unwrap();
		zip.finish().unwrap();
	}

	let cursor = std::io::Cursor::new(buf);
	let mut archive = zip::ZipArchive::new(cursor).expect("valid zip archive");
	let entry = archive.by_name("modrinth.index.json").expect("entry found");

	#[derive(Debug, serde::Deserialize)]
	#[serde(rename_all = "camelCase")]
	#[allow(dead_code)]
	struct TestMrpackManifest {
		#[serde(default)]
		format_version: u32,
		#[serde(default)]
		game: serde_json::Value,
		#[serde(default)]
		version_id: Option<String>,
		#[serde(default)]
		name: Option<String>,
		#[serde(default)]
		files: Vec<TestMrpackFile>,
		#[serde(default)]
		dependencies: std::collections::HashMap<String, serde_json::Value>,
	}

	#[derive(Debug, serde::Deserialize)]
	#[serde(rename_all = "camelCase")]
	#[allow(dead_code)]
	struct TestMrpackFile {
		path: String,
		#[serde(default)]
		downloads: Vec<String>,
		#[serde(default)]
		env: Option<TestMrpackEnv>,
	}

	#[derive(Debug, serde::Deserialize)]
	#[serde(rename_all = "camelCase")]
	#[allow(dead_code)]
	struct TestMrpackEnv {
		#[serde(default)]
		client: Option<String>,
		#[serde(default)]
		server: Option<String>,
	}

	let manifest: TestMrpackManifest = serde_json::from_reader(entry).expect("must parse successfully");

	assert_eq!(manifest.format_version, 1);
	assert_eq!(manifest.game.as_str(), Some("minecraft"));
	assert_eq!(manifest.name.as_deref(), Some("Fabulously Optimized"));
	assert_eq!(manifest.files.len(), 1);
	assert_eq!(manifest.files[0].path, "mods/sodium-fabric-0.6.0.jar");
	assert_eq!(manifest.files[0].downloads.len(), 1);
	assert_eq!(
		manifest.dependencies.get("minecraft").and_then(|v| v.as_str()),
		Some("1.21.1")
	);
	assert_eq!(
		manifest.dependencies.get("fabric-loader").and_then(|v| v.as_str()),
		Some("0.16.5")
	);
}
