use serde::{Deserialize, Serialize};
use std::path::Path;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::cir::Cir;
use crate::converter::compute_bounding_box;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndex {
    pub assets: Vec<AssetEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEntry {
    pub name: String,
    pub tags: Vec<String>,
    pub category: String,
    pub path: String,
    pub part_count: usize,
    pub bounding_box: BoundingBox,
    pub created_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

pub fn load_index(library_dir: &Path) -> anyhow::Result<AssetIndex> {
    let index_path = library_dir.join("index.json");
    if !index_path.exists() {
        return Ok(AssetIndex {
            assets: Vec::new(),
        });
    }
    let content = std::fs::read_to_string(&index_path)?;
    Ok(serde_json::from_str(&content)?)
}

fn save_index(library_dir: &Path, index: &AssetIndex) -> anyhow::Result<()> {
    let index_path = library_dir.join("index.json");
    let content = serde_json::to_string_pretty(&index)?;
    std::fs::write(index_path, content)?;
    Ok(())
}

pub fn search_assets(
    library_dir: &Path,
    query: &str,
    tags: Option<&[String]>,
    limit: usize,
) -> anyhow::Result<Vec<AssetEntry>> {
    let index = load_index(library_dir)?;
    let matcher = SkimMatcherV2::default();

    let mut scored: Vec<(i64, &AssetEntry)> = index
        .assets
        .iter()
        .filter_map(|entry| {
            let name_score = matcher.fuzzy_match(&entry.name, query).unwrap_or(0);
            let tag_score: i64 = entry
                .tags
                .iter()
                .filter_map(|tag| matcher.fuzzy_match(tag, query))
                .max()
                .unwrap_or(0);
            let total_score = name_score.max(tag_score);

            // Filter by required tags if specified
            if let Some(required_tags) = tags {
                let has_all = required_tags.iter().all(|rt| {
                    entry
                        .tags
                        .iter()
                        .any(|et| et.to_lowercase() == rt.to_lowercase())
                });
                if !has_all {
                    return None;
                }
            }

            if total_score > 0 {
                Some((total_score, entry))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));

    Ok(scored
        .into_iter()
        .take(limit)
        .map(|(_, entry)| entry.clone())
        .collect())
}

pub fn save_asset(
    library_dir: &Path,
    cir: &Cir,
    name: &str,
    tags: &[String],
    category: &str,
) -> anyhow::Result<AssetEntry> {
    // Sanitize name and category to prevent path traversal
    let safe_name: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    let safe_category: String = category
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect();

    let category_dir = library_dir.join(&safe_category);
    std::fs::create_dir_all(&category_dir)?;

    let filename = format!("{}.cir.json", safe_name);
    let file_path = category_dir.join(&filename);
    let cir_json = serde_json::to_string_pretty(cir)?;
    std::fs::write(&file_path, cir_json)?;

    let (bb_min, bb_max) = compute_bounding_box(cir);
    let relative_path = format!("{}/{}", safe_category, filename);

    let entry = AssetEntry {
        name: safe_name.clone(),
        tags: tags.to_vec(),
        category: safe_category,
        path: relative_path,
        part_count: cir.parts.len(),
        bounding_box: BoundingBox {
            min: bb_min,
            max: bb_max,
        },
        created_date: chrono::Utc::now().to_rfc3339(),
    };

    let mut index = load_index(library_dir)?;
    // Replace existing entry with same name
    index.assets.retain(|a| a.name != safe_name);
    index.assets.push(entry.clone());
    save_index(library_dir, &index)?;

    Ok(entry)
}

pub fn load_asset_cir(library_dir: &Path, entry: &AssetEntry) -> anyhow::Result<Cir> {
    let path = library_dir.join(&entry.path);
    let content = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&content)?)
}
