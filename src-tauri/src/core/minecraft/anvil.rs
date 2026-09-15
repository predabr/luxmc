use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldChunkInfo {
    pub x: isize,
    pub z: isize,
    pub status: Option<String>,
    pub data_version: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionSummary {
    pub region_file: String,
    pub total_chunks_allocated: usize,
    pub sample_chunks: Vec<WorldChunkInfo>,
}

#[derive(Deserialize, Default)]
struct RawChunkMeta {
    #[serde(rename = "Status")]
    status: Option<String>,
    #[serde(rename = "DataVersion")]
    data_version: Option<i32>,
}

pub fn inspect_region_file(mca_path: &Path) -> Option<RegionSummary> {
    let file = std::fs::File::open(mca_path).ok()?;
    let mmap = unsafe { memmap2::MmapOptions::new().map(&file).ok()? };
    let cursor = std::io::Cursor::new(&mmap[..]);

    let mut region = fastanvil::Region::from_stream(cursor).ok()?;
    let mut total_chunks = 0;
    let mut sample_chunks = Vec::new();

    for z in 0..32 {
        for x in 0..32 {
            if let Ok(Some(chunk_bytes)) = region.read_chunk(x, z) {
                total_chunks += 1;
                if sample_chunks.len() < 4 {
                    let meta: RawChunkMeta = fastnbt::from_bytes(&chunk_bytes).unwrap_or_default();

                    sample_chunks.push(WorldChunkInfo {
                        x: x as isize,
                        z: z as isize,
                        status: meta.status,
                        data_version: meta.data_version,
                    });
                }
            }
        }
    }

    Some(RegionSummary {
        region_file: mca_path.file_name()?.to_string_lossy().to_string(),
        total_chunks_allocated: total_chunks,
        sample_chunks,
    })
}
