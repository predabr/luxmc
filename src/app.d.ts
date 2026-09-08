declare namespace App {}

interface ImportMetaEnv {
	readonly PUBLIC_TAURI?: string;
}

interface ImportMeta {
	readonly env: ImportMetaEnv;
}
