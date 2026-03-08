use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use serde_json::{json, Value};

mod assembler;
mod asset_library;
mod cir;
mod converter;
mod scale_config;
mod scene_reader;
mod validator;

struct Server {
    base_dir: PathBuf,
}

impl Server {
    fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    fn config_dir(&self) -> PathBuf {
        self.base_dir.join("config")
    }

    fn library_dir(&self) -> PathBuf {
        self.base_dir.join("assets").join("library")
    }

    fn generated_dir(&self) -> PathBuf {
        self.base_dir.join("assets").join("generated")
    }

    fn run(&self) {
        let stdin = io::stdin();
        let stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) if !l.trim().is_empty() => l,
                Ok(_) => continue,
                Err(_) => break,
            };

            let request: Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if let Some(response) = self.handle_message(&request) {
                let mut out = stdout.lock();
                let _ = writeln!(out, "{}", serde_json::to_string(&response).unwrap());
                let _ = out.flush();
            }
        }
    }

    fn handle_message(&self, request: &Value) -> Option<Value> {
        let id = request.get("id").cloned()?; // notifications have no id
        let method = request
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("");
        let params = request.get("params").cloned().unwrap_or(json!({}));

        let result = match method {
            "initialize" => Ok(self.handle_initialize()),
            "notifications/initialized" => return None,
            "tools/list" => Ok(self.handle_tools_list()),
            "tools/call" => self.handle_tools_call(&params),
            _ => Ok(json!({})),
        };

        Some(match result {
            Ok(val) => json!({"jsonrpc": "2.0", "id": id, "result": val}),
            Err(msg) => json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32000, "message": msg}}),
        })
    }

    fn handle_initialize(&self) -> Value {
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": {
                "name": "roblox-asset-pipeline",
                "version": "0.1.0"
            }
        })
    }

    fn handle_tools_list(&self) -> Value {
        json!({ "tools": self.tool_definitions() })
    }

    fn handle_tools_call(&self, params: &Value) -> Result<Value, String> {
        let name = params
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or("Missing tool name")?;
        let args = params.get("arguments").cloned().unwrap_or(json!({}));

        let result = match name {
            "validate_geometry" => self.tool_validate_geometry(&args),
            "convert_to_model" => self.tool_convert_to_model(&args),
            "convert_and_save" => self.tool_convert_and_save(&args),
            "search_assets" => self.tool_search_assets(&args),
            "save_asset" => self.tool_save_asset(&args),
            "read_scene_context" => self.tool_read_scene_context(&args),
            "assemble_scene" => self.tool_assemble_scene(&args),
            "get_scale_config" => self.tool_get_scale_config(&args),
            "get_palette" => self.tool_get_palette(&args),
            _ => Err(format!("Unknown tool: {}", name)),
        };

        Ok(match result {
            Ok(text) => json!({"content": [{"type": "text", "text": text}]}),
            Err(msg) => json!({"content": [{"type": "text", "text": msg}], "isError": true}),
        })
    }

    // ── Tool definitions ──

    fn tool_definitions(&self) -> Value {
        json!([
            {
                "name": "validate_geometry",
                "description": "Validate geometry of a CIR asset. Checks underground parts, overlaps, floating, scale violations, part budget, palette, and name collisions. Returns errors with fix suggestions.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "cir_json": {"type": "string", "description": "CIR JSON string"},
                        "palette": {"type": "string", "description": "Palette name to check colors against"},
                        "max_parts": {"type": "integer", "description": "Max part count (default 30)"}
                    },
                    "required": ["cir_json"]
                }
            },
            {
                "name": "convert_to_model",
                "description": "Convert CIR to Rojo .model.json and write to assets/generated/. Validates first — rejects on errors. File auto-syncs to Studio via rojo serve.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "cir_json": {"type": "string", "description": "CIR JSON string"},
                        "output_name": {"type": "string", "description": "Filename without extension (e.g. 'medieval_barn')"}
                    },
                    "required": ["cir_json", "output_name"]
                }
            },
            {
                "name": "search_assets",
                "description": "Search the asset library by name and tags. Returns matching assets with metadata (part count, bounding box, tags).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "Search query"},
                        "tags": {"type": "array", "items": {"type": "string"}, "description": "Required tags filter"},
                        "limit": {"type": "integer", "description": "Max results (default 5)"}
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "save_asset",
                "description": "Save a validated CIR asset to the library for future reuse.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "cir_json": {"type": "string", "description": "CIR JSON string"},
                        "name": {"type": "string", "description": "Asset name"},
                        "tags": {"type": "array", "items": {"type": "string"}, "description": "Tags for search"},
                        "category": {"type": "string", "description": "Category: nature, structures, farm, decorations"}
                    },
                    "required": ["cir_json", "name", "tags", "category"]
                }
            },
            {
                "name": "read_scene_context",
                "description": "Read generated assets and return spatial summary with positions, bounding boxes, and open areas for placement.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "assemble_scene",
                "description": "Assemble multiple assets into one scene .model.json. Takes asset refs (library names) or inline CIR with positions.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "spec_json": {"type": "string", "description": "Assembly spec JSON: {name, assets: [{ref?, cir?, position, rotation}]}"}
                    },
                    "required": ["spec_json"]
                }
            },
            {
                "name": "get_scale_config",
                "description": "Get canonical scale dimensions (humanoid height, door size, fence height, etc.) for consistent asset sizing.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "get_palette",
                "description": "Get a named color palette (medieval, nature, farm, industrial) for consistent coloring.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string", "description": "Palette name"}
                    },
                    "required": ["name"]
                }
            },
            {
                "name": "convert_and_save",
                "description": "Validate, convert CIR to .model.json, AND save to asset library — all in one call. Use this instead of separate validate/convert/save calls.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "cir_json": {"type": "string", "description": "CIR JSON string"},
                        "output_name": {"type": "string", "description": "Filename without extension (e.g. 'medieval_house')"},
                        "tags": {"type": "array", "items": {"type": "string"}, "description": "Tags for library search"},
                        "category": {"type": "string", "description": "Category: nature, structures, farm, decorations"}
                    },
                    "required": ["cir_json", "output_name", "tags", "category"]
                }
            }
        ])
    }

    // ── Tool implementations ──

    fn tool_validate_geometry(&self, args: &Value) -> Result<String, String> {
        let cir_json = args
            .get("cir_json")
            .and_then(|v| v.as_str())
            .ok_or("Missing cir_json")?;
        let cir =
            cir::Cir::from_json(cir_json).map_err(|e| format!("Invalid CIR JSON: {}", e))?;

        let scale = scale_config::load_scale_config(&self.config_dir())
            .map_err(|e| format!("Failed to load scale config: {}", e))?;

        let palette = if let Some(name) = args.get("palette").and_then(|v| v.as_str()) {
            Some(
                scale_config::load_palette(&self.config_dir(), name)
                    .map_err(|e| format!("Failed to load palette: {}", e))?,
            )
        } else {
            None
        };

        let max_parts = args
            .get("max_parts")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        let opts = validator::ValidateOptions {
            scale: &scale,
            palette: palette.as_ref(),
            max_parts,
        };

        let result = validator::validate(&cir, &opts);
        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    fn tool_convert_to_model(&self, args: &Value) -> Result<String, String> {
        let cir_json = args
            .get("cir_json")
            .and_then(|v| v.as_str())
            .ok_or("Missing cir_json")?;
        let output_name = args
            .get("output_name")
            .and_then(|v| v.as_str())
            .ok_or("Missing output_name")?;

        // Sanitize filename
        let safe_name: String = output_name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .collect();

        let cir =
            cir::Cir::from_json(cir_json).map_err(|e| format!("Invalid CIR JSON: {}", e))?;

        // Validate first
        let scale = scale_config::load_scale_config(&self.config_dir())
            .map_err(|e| format!("Failed to load scale config: {}", e))?;

        let opts = validator::ValidateOptions {
            scale: &scale,
            palette: None,
            max_parts: None,
        };

        let validation = validator::validate(&cir, &opts);
        if !validation.valid {
            return Err(format!(
                "Validation failed:\n{}",
                serde_json::to_string_pretty(&validation).unwrap()
            ));
        }

        let output_path = self
            .generated_dir()
            .join(format!("{}.model.json", safe_name));
        let path = converter::write_model_json(&cir, &output_path)
            .map_err(|e| format!("Failed to write model: {}", e))?;

        let (bb_min, bb_max) = converter::compute_bounding_box(&cir);

        let result = json!({
            "path": path,
            "part_count": cir.parts.len(),
            "bounding_box": {"min": bb_min, "max": bb_max},
            "warnings": validation.warnings,
        });

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    fn tool_search_assets(&self, args: &Value) -> Result<String, String> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or("Missing query")?;
        let tags: Option<Vec<String>> = args.get("tags").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });
        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(5) as usize;

        let results =
            asset_library::search_assets(&self.library_dir(), query, tags.as_deref(), limit)
                .map_err(|e| format!("Search failed: {}", e))?;

        if results.is_empty() {
            Ok("No matching assets found.".to_string())
        } else {
            serde_json::to_string_pretty(&results).map_err(|e| e.to_string())
        }
    }

    fn tool_save_asset(&self, args: &Value) -> Result<String, String> {
        let cir_json = args
            .get("cir_json")
            .and_then(|v| v.as_str())
            .ok_or("Missing cir_json")?;
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing name")?;
        let tags: Vec<String> = args
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let category = args
            .get("category")
            .and_then(|v| v.as_str())
            .ok_or("Missing category")?;

        let cir =
            cir::Cir::from_json(cir_json).map_err(|e| format!("Invalid CIR JSON: {}", e))?;

        let entry = asset_library::save_asset(&self.library_dir(), &cir, name, &tags, category)
            .map_err(|e| format!("Save failed: {}", e))?;

        serde_json::to_string_pretty(&entry).map_err(|e| e.to_string())
    }

    fn tool_convert_and_save(&self, args: &Value) -> Result<String, String> {
        let cir_json = args
            .get("cir_json")
            .and_then(|v| v.as_str())
            .ok_or("Missing cir_json")?;
        let output_name = args
            .get("output_name")
            .and_then(|v| v.as_str())
            .ok_or("Missing output_name")?;
        let tags: Vec<String> = args
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let category = args
            .get("category")
            .and_then(|v| v.as_str())
            .ok_or("Missing category")?;

        let safe_name: String = output_name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .collect();

        let cir =
            cir::Cir::from_json(cir_json).map_err(|e| format!("Invalid CIR JSON: {}", e))?;

        // Validate
        let scale = scale_config::load_scale_config(&self.config_dir())
            .map_err(|e| format!("Failed to load scale config: {}", e))?;
        let opts = validator::ValidateOptions {
            scale: &scale,
            palette: None,
            max_parts: None,
        };
        let validation = validator::validate(&cir, &opts);
        if !validation.valid {
            return Err(format!(
                "Validation failed:\n{}",
                serde_json::to_string_pretty(&validation).unwrap()
            ));
        }

        // Convert and write .model.json
        let output_path = self
            .generated_dir()
            .join(format!("{}.model.json", safe_name));
        let path = converter::write_model_json(&cir, &output_path)
            .map_err(|e| format!("Failed to write model: {}", e))?;

        // Save to library
        let entry =
            asset_library::save_asset(&self.library_dir(), &cir, output_name, &tags, category)
                .map_err(|e| format!("Library save failed: {}", e))?;

        let (bb_min, bb_max) = converter::compute_bounding_box(&cir);

        let result = json!({
            "path": path,
            "part_count": cir.parts.len(),
            "bounding_box": {"min": bb_min, "max": bb_max},
            "warnings": validation.warnings,
            "library_entry": entry,
        });

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    fn tool_read_scene_context(&self, _args: &Value) -> Result<String, String> {
        let ctx = scene_reader::read_scene(&self.generated_dir())
            .map_err(|e| format!("Failed to read scene: {}", e))?;
        Ok(scene_reader::format_scene_context(&ctx))
    }

    fn tool_assemble_scene(&self, args: &Value) -> Result<String, String> {
        let spec_json = args
            .get("spec_json")
            .and_then(|v| v.as_str())
            .ok_or("Missing spec_json")?;
        let spec: assembler::AssemblySpec =
            serde_json::from_str(spec_json).map_err(|e| format!("Invalid assembly spec: {}", e))?;

        let output_path = self
            .generated_dir()
            .join("scenes")
            .join(format!("{}.model.json", spec.name));

        let result =
            assembler::assemble_scene(&spec, &self.library_dir(), &output_path)
                .map_err(|e| format!("Assembly failed: {}", e))?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    fn tool_get_scale_config(&self, _args: &Value) -> Result<String, String> {
        let config = scale_config::load_scale_config(&self.config_dir())
            .map_err(|e| format!("Failed to load scale config: {}", e))?;
        serde_json::to_string_pretty(&config).map_err(|e| e.to_string())
    }

    fn tool_get_palette(&self, args: &Value) -> Result<String, String> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing palette name")?;
        let palette = scale_config::load_palette(&self.config_dir(), name)
            .map_err(|e| format!("{}", e))?;
        serde_json::to_string_pretty(&palette).map_err(|e| e.to_string())
    }
}

fn main() {
    let base_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("Cannot get current directory"));

    let server = Server::new(base_dir);
    server.run();
}
