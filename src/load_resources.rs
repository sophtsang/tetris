//! Contains software to load resources and configuration into our resource types.

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::resources::{Config, Offset, TetrominoConfig, Tetrominoes, Wallkick};

/// Need to parse `TetrominoConfig` in a form with `wall_kick` as a
/// `String` first until wall kick config can be parsed.
#[derive(Deserialize, Debug)]
struct RawTetrominoConfig {
    states: [[Offset; 4]; 4],
    wall_kick: String,
}

impl TetrominoConfig {
    /// Resolves a raw config by replacing its wall kick table name with
    /// the actual table from `wall_kicks`.
    fn from_raw(
        name: &str,
        raw: RawTetrominoConfig,
        wall_kicks: &HashMap<String, Wallkick>,
    ) -> Result<Self, Box<dyn Error>> {
        let wall_kick = wall_kicks.get(&raw.wall_kick).cloned().ok_or_else(|| {
            format!(
                "\"{name}\" uses wall kick table \"{}\", which is not in wall_kicks.json",
                raw.wall_kick
            )
        })?;
        Ok(Self {
            states: raw.states,
            wall_kick,
        })
    }
}

/// Deserialize JSON file to object.
fn read_json<T: DeserializeOwned>(dir: &Path, name: &str) -> Result<T, Box<dyn Error>> {
    let path = dir.join(name);
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("could not read {}: {e}", path.display()))?;
    let value = serde_json::from_str(&text)
        .map_err(|e| format!("could not parse {}: {e}", path.display()))?;
    Ok(value)
}

/// Loads `config.json`, `tetrominoes.json` and `wall_kicks.json` from `dir`.
pub fn load_resources(dir: &Path) -> Result<(Config, Tetrominoes), Box<dyn Error>> {
    let config: Config = read_json(dir, "config.json")?;
    let raw: HashMap<String, RawTetrominoConfig> = read_json(dir, "tetrominoes.json")?;
    let wall_kicks: HashMap<String, Wallkick> = read_json(dir, "wall_kicks.json")?;
    let configs = raw
        .into_iter()
        .map(|(name, raw)| {
            let config = TetrominoConfig::from_raw(&name, raw, &wall_kicks)?;
            Ok((name, config))
        })
        .collect::<Result<HashMap<_, _>, Box<dyn Error>>>()?;
    let tetrominoes = Tetrominoes::new(configs)?;
    Ok((config, tetrominoes))
}
