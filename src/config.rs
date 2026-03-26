use std::fs;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpriteSource {
    Sprites,
    Home,
    #[serde(alias = "official-artwork")]
    #[default]
    OfficialArtwork,
    Showdown,
    #[serde(alias = "dream-world")]
    DreamWorld,
}

impl SpriteSource {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "sprites" => Some(Self::Sprites),
            "home" => Some(Self::Home),
            "official_artwork" | "official-artwork" => Some(Self::OfficialArtwork),
            "showdown" => Some(Self::Showdown),
            "dream_world" | "dream-world" => Some(Self::DreamWorld),
            _ => None,
        }
    }

    pub fn variants() -> &'static str {
        "sprites|home|official_artwork|showdown|dream_world"
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpriteGender {
    #[default]
    Default,
    Female,
}

impl SpriteGender {

    pub fn is_female(self) -> bool {
        matches!(self, Self::Female)
    }
}

#[derive(Clone, Debug)]
pub struct SpriteRequest {
    pub source: SpriteSource,
    pub shiny: bool,
    pub gender: SpriteGender,
    pub generation: Option<String>,
    pub version_group: Option<String>,
    pub animated: bool,
}

#[derive(Default, Deserialize)]
struct RawConfig {
    cache: Option<CacheField>,
    sprite_source: Option<SpriteSource>,
    sprite_gender: Option<SpriteGender>,
    sprite_generation: Option<String>,
    sprite_version_group: Option<String>,
    sprite_animated: Option<bool>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum CacheField {
    Enabled(bool),
    Section { enabled: bool },
}

#[derive(Default)]
pub struct AppConfig {
    pub cache_enabled: bool,
    pub sprite_source: Option<SpriteSource>,
    pub sprite_gender: Option<SpriteGender>,
    pub sprite_generation: Option<String>,
    pub sprite_version_group: Option<String>,
    pub sprite_animated: Option<bool>,
}

impl AppConfig {
    pub fn default_sprite_request(&self, shiny: bool) -> SpriteRequest {
        SpriteRequest {
            source: self.sprite_source.unwrap_or_default(),
            shiny,
            gender: self.sprite_gender.unwrap_or_default(),
            generation: self.sprite_generation.clone(),
            version_group: self.sprite_version_group.clone(),
            animated: self.sprite_animated.unwrap_or(false),
        }
    }
}

pub fn load_config() -> Result<AppConfig, String> {
    let Some(config_dir) = dirs::config_dir() else {
        return Ok(AppConfig::default());
    };

    let config_path = config_dir.join("ferrisdex").join("config.toml");
    if !config_path.exists() {
        return Ok(AppConfig::default());
    }

    let raw = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config '{}': {}", config_path.display(), e))?;
    let parsed: RawConfig = toml::from_str(&raw)
        .map_err(|e| format!("Failed to parse config '{}': {}", config_path.display(), e))?;

    let cache_enabled = match parsed.cache {
        Some(CacheField::Enabled(value)) => value,
        Some(CacheField::Section { enabled }) => enabled,
        None => false,
    };

    Ok(AppConfig {
        cache_enabled,
        sprite_source: parsed.sprite_source,
        sprite_gender: parsed.sprite_gender,
        sprite_generation: parsed.sprite_generation,
        sprite_version_group: parsed.sprite_version_group,
        sprite_animated: parsed.sprite_animated,
    })
}

#[cfg(test)]
mod tests {
    use super::{CacheField, RawConfig, SpriteGender, SpriteSource};

    #[test]
    fn parses_all_sprite_config_keys() {
        let raw = r#"
cache = true
sprite_source = "home"
sprite_gender = "female"
sprite_generation = "generation-v"
sprite_version_group = "black-white"
sprite_animated = true
"#;

        let parsed: RawConfig = toml::from_str(raw).expect("config should parse");

        assert!(matches!(parsed.cache, Some(CacheField::Enabled(true))));
        assert_eq!(parsed.sprite_source, Some(SpriteSource::Home));
        assert_eq!(parsed.sprite_gender, Some(SpriteGender::Female));
        assert_eq!(parsed.sprite_generation.as_deref(), Some("generation-v"));
        assert_eq!(parsed.sprite_version_group.as_deref(), Some("black-white"));
        assert_eq!(parsed.sprite_animated, Some(true));
    }

    #[test]
    fn parses_cache_section_syntax() {
        let raw = r#"
[cache]
enabled = true
"#;

        let parsed: RawConfig = toml::from_str(raw).expect("config should parse");
        assert!(matches!(parsed.cache, Some(CacheField::Section { enabled: true })));
    }
}

