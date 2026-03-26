use std::env;
mod api;
mod cli;
mod config;
mod display;
mod terminal;

use api::{download_sprite, fetch_display_name, fetch_pokemon, get_sprite};
use cli::parse_args;
use config::{AppConfig, load_config};
use display::build_info_lines;
use terminal::render_pokedex_view;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let cli = parse_args(args).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    let app_config = load_config().unwrap_or_else(|e| {
        eprintln!("Warning: {}", e);
        AppConfig::default()
    });

    let pokemon_name = cli.pokemon_name;
    let cache_enabled = cli.cache_enabled || app_config.cache_enabled;

    let mut sprite_request = app_config.default_sprite_request(cli.shiny);
    if let Some(source) = cli.sprite_source {
        sprite_request.source = source;
    }
    if let Some(gender) = cli.sprite_gender {
        sprite_request.gender = gender;
    }
    if let Some(generation) = cli.sprite_generation {
        sprite_request.generation = Some(generation);
    }
    if let Some(version_group) = cli.sprite_version_group {
        sprite_request.version_group = Some(version_group);
    }
    if let Some(animated) = cli.sprite_animated {
        sprite_request.animated = animated;
    }

    let pokemon = fetch_pokemon(&pokemon_name, cache_enabled).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let sprite_url = get_sprite(&pokemon, &sprite_request).unwrap_or_else(|| {
        eprintln!(
            "Error: No sprite available for '{}' with source '{:?}'.",
            pokemon_name, sprite_request.source
        );
        std::process::exit(1);
    });

    // Step 2: fire sprite download and display name fetch in parallel
    let image_thread = std::thread::spawn(move || download_sprite(&sprite_url, cache_enabled));

    let species_query = pokemon.name.clone();
    let display_name_thread =
        std::thread::spawn(move || fetch_display_name(&species_query, cache_enabled));

    // Join both
    let img = image_thread.join().unwrap().unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let display_name = display_name_thread
        .join()
        .unwrap()
        .unwrap_or_else(|| pokemon.name.clone());

    let info_lines = build_info_lines(&pokemon, &display_name);
    render_pokedex_view(&img, &info_lines)?;

    Ok(())
}
