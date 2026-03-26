use crate::config::{SpriteGender, SpriteSource};

#[derive(Debug)]
pub struct CliOptions {
    pub pokemon_name: String,
    pub shiny: bool,
    pub cache_enabled: bool,
    pub sprite_source: Option<SpriteSource>,
    pub sprite_gender: Option<SpriteGender>,
    pub sprite_generation: Option<String>,
    pub sprite_version_group: Option<String>,
    pub sprite_animated: Option<bool>,
}

impl CliOptions {
    pub fn usage(bin_name: &str) -> String {
        format!(
            "Usage: {} <pokemon_name_or_id> [shiny] [-c|--cache] [-s|--sprite-source <{}>] [--female|--male] [-g|--generation <i..ix|generation-i..generation-ix>] [--version-group <group>] [--animated]",
            bin_name,
            SpriteSource::variants()
        )
    }
}

pub fn parse_args(args: Vec<String>) -> Result<CliOptions, String> {
    let bin_name = args
        .first()
        .cloned()
        .unwrap_or_else(|| "ferrisdex".to_string());

    let mut pokemon_name: Option<String> = None;
    let mut shiny = false;
    let mut cache_enabled = false;
    let mut sprite_source: Option<SpriteSource> = None;
    let mut sprite_gender: Option<SpriteGender> = None;
    let mut sprite_generation: Option<String> = None;
    let mut sprite_version_group: Option<String> = None;
    let mut sprite_animated: Option<bool> = None;
    let mut pending_sprite_source = false;
    let mut pending_generation = false;
    let mut pending_version_group = false;

    for arg in args.into_iter().skip(1) {
        if pending_sprite_source {
            sprite_source = Some(parse_sprite_source(&arg, &bin_name)?);
            pending_sprite_source = false;
            continue;
        }
        if pending_generation {
            sprite_generation = Some(parse_generation(&arg, &bin_name)?);
            pending_generation = false;
            continue;
        }
        if pending_version_group {
            if arg.is_empty() {
                return Err(format!(
                    "Missing value for --version-group.\n{}",
                    CliOptions::usage(&bin_name)
                ));
            }
            sprite_version_group = Some(arg);
            pending_version_group = false;
            continue;
        }

        match arg.as_str() {
            "-c" | "--cache" => cache_enabled = true,
            "-s" | "--sprite-source" => pending_sprite_source = true,
            "--female" => sprite_gender = Some(SpriteGender::Female),
            "--male" => sprite_gender = Some(SpriteGender::Default),
            "-g" | "--generation" => pending_generation = true,
            "--version-group" => pending_version_group = true,
            "--animated" => sprite_animated = Some(true),
            "shiny" if pokemon_name.is_some() => shiny = true,
            _ if arg.starts_with("--sprite-source=") => {
                let value = &arg["--sprite-source=".len()..];
                sprite_source = Some(parse_sprite_source(value, &bin_name)?);
            }
            _ if arg.starts_with("--generation=") => {
                let value = &arg["--generation=".len()..];
                sprite_generation = Some(parse_generation(value, &bin_name)?);
            }
            _ if arg.starts_with("--version-group=") => {
                let value = &arg["--version-group=".len()..];
                if value.is_empty() {
                    return Err(format!(
                        "Missing value for --version-group.\n{}",
                        CliOptions::usage(&bin_name)
                    ));
                }
                sprite_version_group = Some(value.to_string());
            }
            _ if arg.starts_with('-') => {
                return Err(format!(
                    "Unknown flag '{}'.\n{}",
                    arg,
                    CliOptions::usage(&bin_name)
                ));
            }
            _ if pokemon_name.is_none() => pokemon_name = Some(arg),
            _ => {
                return Err(format!(
                    "Unexpected argument '{}'.\n{}",
                    arg,
                    CliOptions::usage(&bin_name)
                ));
            }
        }
    }

    if pending_sprite_source || pending_generation || pending_version_group {
        return Err(format!(
            "Missing value for a sprite option.\n{}",
            CliOptions::usage(&bin_name)
        ));
    }

    let Some(pokemon_name) = pokemon_name else {
        return Err(CliOptions::usage(&bin_name));
    };

    Ok(CliOptions {
        pokemon_name,
        shiny,
        cache_enabled,
        sprite_source,
        sprite_gender,
        sprite_generation,
        sprite_version_group,
        sprite_animated,
    })
}

fn parse_sprite_source(value: &str, bin_name: &str) -> Result<SpriteSource, String> {
    SpriteSource::parse(value).ok_or_else(|| {
        format!(
            "Unknown sprite source '{}'. Expected one of: {}.\n{}",
            value,
            SpriteSource::variants(),
            CliOptions::usage(bin_name)
        )
    })
}

fn parse_generation(value: &str, bin_name: &str) -> Result<String, String> {
    let normalized = value.to_ascii_lowercase();
    let suffix = if let Some(stripped) = normalized.strip_prefix("generation-") {
        stripped
    } else {
        normalized.as_str()
    };
    let valid = ["i", "ii", "iii", "iv", "v", "vi", "vii", "viii", "ix"];
    if valid.contains(&suffix) {
        Ok(format!("generation-{}", suffix))
    } else {
        Err(format!(
            "Unknown generation '{}'. Expected one of i..ix or generation-i..generation-ix.\n{}",
            value,
            CliOptions::usage(bin_name)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::parse_args;

    #[test]
    fn parses_cache_and_shiny() {
        let args = vec![
            "ferrisdex".to_string(),
            "pikachu".to_string(),
            "--cache".to_string(),
            "shiny".to_string(),
        ];

        let parsed = parse_args(args).expect("args should parse");
        assert_eq!(parsed.pokemon_name, "pikachu");
        assert!(parsed.shiny);
        assert!(parsed.cache_enabled);
        assert_eq!(parsed.sprite_source, None);
        assert_eq!(parsed.sprite_gender, None);
    }

    #[test]
    fn parses_sprite_source_equals_syntax() {
        let args = vec![
            "ferrisdex".to_string(),
            "pikachu".to_string(),
            "--sprite-source=showdown".to_string(),
        ];

        let parsed = parse_args(args).expect("args should parse");
        assert_eq!(parsed.pokemon_name, "pikachu");
        assert_eq!(
            parsed.sprite_source,
            Some(crate::config::SpriteSource::Showdown)
        );
    }

    #[test]
    fn parses_generation_and_gender() {
        let args = vec![
            "ferrisdex".to_string(),
            "pikachu".to_string(),
            "-g".to_string(),
            "ix".to_string(),
            "--female".to_string(),
            "--animated".to_string(),
        ];

        let parsed = parse_args(args).expect("args should parse");
        assert_eq!(parsed.sprite_generation.as_deref(), Some("generation-ix"));
        assert_eq!(
            parsed.sprite_gender,
            Some(crate::config::SpriteGender::Female)
        );
        assert_eq!(parsed.sprite_animated, Some(true));
    }

    #[test]
    fn rejects_removed_gender_and_static_flags() {
        let args = vec![
            "ferrisdex".to_string(),
            "pikachu".to_string(),
            "--gender".to_string(),
            "female".to_string(),
        ];
        let err = parse_args(args).expect_err("args should fail");
        assert!(err.contains("Unknown flag '--gender'"));

        let args = vec![
            "ferrisdex".to_string(),
            "pikachu".to_string(),
            "--static".to_string(),
        ];
        let err = parse_args(args).expect_err("args should fail");
        assert!(err.contains("Unknown flag '--static'"));
    }

    #[test]
    fn gender_flags_last_wins() {
        let args = vec![
            "ferrisdex".to_string(),
            "pikachu".to_string(),
            "--female".to_string(),
            "--male".to_string(),
        ];
        let parsed = parse_args(args).expect("args should parse");
        assert_eq!(
            parsed.sprite_gender,
            Some(crate::config::SpriteGender::Default)
        );

        let args = vec![
            "ferrisdex".to_string(),
            "pikachu".to_string(),
            "--male".to_string(),
            "--female".to_string(),
        ];
        let parsed = parse_args(args).expect("args should parse");
        assert_eq!(
            parsed.sprite_gender,
            Some(crate::config::SpriteGender::Female)
        );
    }

    #[test]
    fn repeated_generation_flags_last_wins() {
        let args = vec![
            "ferrisdex".to_string(),
            "pikachu".to_string(),
            "-g".to_string(),
            "i".to_string(),
            "-g".to_string(),
            "ii".to_string(),
        ];

        let parsed = parse_args(args).expect("args should parse");
        assert_eq!(parsed.sprite_generation.as_deref(), Some("generation-ii"));
    }

    #[test]
    fn rejects_invalid_sprite_source() {
        let args = vec![
            "ferrisdex".to_string(),
            "pikachu".to_string(),
            "--sprite-source=nope".to_string(),
        ];

        let err = parse_args(args).expect_err("args should fail");
        assert!(err.contains("Unknown sprite source"));
    }

    #[test]
    fn rejects_unknown_flag() {
        let args = vec!["ferrisdex".to_string(), "--bad".to_string()];
        let err = parse_args(args).expect_err("args should fail");
        assert!(err.contains("Unknown flag"));
    }

    #[test]
    fn allows_shiny_as_pokemon_name() {
        let args = vec!["ferrisdex".to_string(), "shiny".to_string()];
        let parsed = parse_args(args).expect("args should parse");
        assert_eq!(parsed.pokemon_name, "shiny");
        assert!(!parsed.shiny);
    }
}
