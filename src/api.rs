use std::io;
use std::path::PathBuf;

use http_cache_reqwest::{
    CACacheManager as HttpCACacheManager, Cache as HttpCacheMiddleware, CacheMode as HttpCacheMode,
    HttpCache, HttpCacheOptions,
};
use image::{DynamicImage, ImageReader};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use rustemon::client::{
    CACacheManager as RustemonCACacheManager, CacheMode as RustemonCacheMode, RustemonClientBuilder,
};
use rustemon::model::pokemon::Pokemon;

use crate::config::{SpriteRequest, SpriteSource};

pub fn fetch_pokemon(query: &str, cache_enabled: bool) -> Result<Pokemon, String> {
    run_async(async {
        let client = build_rustemon_client(cache_enabled)?;
        let result = if let Ok(id) = query.parse::<i64>() {
            rustemon::pokemon::pokemon::get_by_id(id, &client).await
        } else {
            rustemon::pokemon::pokemon::get_by_name(query, &client).await
        };

        result.map_err(|e| format!("Failed to fetch pokemon '{}': {}", query, e))
    })
}

pub fn fetch_display_name(species_name: &str, cache_enabled: bool) -> Option<String> {
    run_async(async {
        let client = build_rustemon_client(cache_enabled)?;
        let species = rustemon::pokemon::pokemon_species::get_by_name(species_name, &client)
            .await
            .map_err(|e| format!("Failed to fetch species '{}': {}", species_name, e))?;

        Ok(species
            .names
            .into_iter()
            .find(|n| n.language.name == "en")
            .map(|n| n.name))
    })
    .ok()
    .flatten()
}

pub fn get_sprite(pokemon: &Pokemon, request: &SpriteRequest) -> Option<String> {
    if let Some(generation) = request.generation.as_deref() {
        return Some(generation_sprite_url(
            pokemon.id,
            generation,
            request.version_group.as_deref(),
            request.shiny,
            request.gender.is_female(),
            request.animated,
        ));
    }

    for source in fallback_sources(request.source) {
        if let Some(url) = sprite_from_source(pokemon, request, source) {
            return Some(url);
        }
    }

    None
}

fn sprite_from_source(
    pokemon: &Pokemon,
    request: &SpriteRequest,
    source: SpriteSource,
) -> Option<String> {
    match source {
        SpriteSource::OfficialArtwork => {
            let default = &pokemon.sprites.other.official_artwork.front_default;
            if request.shiny {
                derived_official_artwork_shiny_url(default).or_else(|| default.clone())
            } else {
                default.clone()
            }
        }
        SpriteSource::Sprites => sprite_pair(
            &pokemon.sprites.front_default,
            &pokemon.sprites.front_shiny,
            &pokemon.sprites.front_female,
            &pokemon.sprites.front_shiny_female,
            request.shiny,
            request.gender.is_female(),
        ),
        SpriteSource::DreamWorld => pokemon
            .sprites
            .other
            .dream_world
            .front_female
            .clone()
            .filter(|_| request.gender.is_female())
            .or_else(|| pokemon.sprites.other.dream_world.front_default.clone()),
        SpriteSource::Home => sprite_pair(
            &pokemon.sprites.other.home.front_default,
            &pokemon.sprites.other.home.front_shiny,
            &pokemon.sprites.other.home.front_female,
            &pokemon.sprites.other.home.front_shiny_female,
            request.shiny,
            request.gender.is_female(),
        ),
        SpriteSource::Showdown => Some(showdown_url(
            pokemon.id,
            request.shiny,
            request.gender.is_female(),
        )),
    }
}

fn fallback_sources(primary: SpriteSource) -> Vec<SpriteSource> {
    let ordered = [
        SpriteSource::OfficialArtwork,
        SpriteSource::Sprites,
        SpriteSource::DreamWorld,
        SpriteSource::Home,
        SpriteSource::Showdown,
    ];

    let mut sources = Vec::with_capacity(ordered.len());
    if !sources.contains(&primary) {
        sources.push(primary);
    }
    for source in ordered {
        if !sources.contains(&source) {
            sources.push(source);
        }
    }
    sources
}

fn sprite_pair(
    default: &Option<String>,
    shiny_url: &Option<String>,
    female_url: &Option<String>,
    shiny_female_url: &Option<String>,
    shiny: bool,
    female: bool,
) -> Option<String> {
    match (female, shiny) {
        (true, true) => shiny_female_url
            .clone()
            .or_else(|| female_url.clone())
            .or_else(|| shiny_url.clone())
            .or_else(|| default.clone()),
        (true, false) => female_url.clone().or_else(|| default.clone()),
        (false, true) => shiny_url.clone().or_else(|| default.clone()),
        (false, false) => default.clone(),
    }
}

fn derived_official_artwork_shiny_url(front_default: &Option<String>) -> Option<String> {
    let url = front_default.as_ref()?;
    if url.contains("/other/official-artwork/") {
        Some(url.replace("/other/official-artwork/", "/other/official-artwork/shiny/"))
    } else {
        None
    }
}

fn showdown_url(id: i64, shiny: bool, female: bool) -> String {
    let mut path = String::from(
        "https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon/other/showdown",
    );
    if shiny {
        path.push_str("/shiny");
    }
    if female {
        path.push_str("/female");
    }
    path.push('/');
    path.push_str(&id.to_string());
    path.push_str(".gif");
    path
}

fn generation_sprite_url(
    id: i64,
    generation: &str,
    version_group: Option<&str>,
    shiny: bool,
    female: bool,
    animated: bool,
) -> String {
    let version = version_group
        .map(str::to_string)
        .unwrap_or_else(|| default_version_group(generation).to_string());
    let (shiny, female, animated) =
        normalize_generation_variant(generation, version.as_str(), shiny, female, animated);

    let mut path = format!(
        "https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon/versions/{}/{}/",
        generation, version
    );
    if animated {
        path.push_str("animated/");
    }
    if shiny {
        path.push_str("shiny/");
    }
    if female {
        path.push_str("female/");
    }
    path.push_str(&id.to_string());
    path.push_str(if animated { ".gif" } else { ".png" });
    path
}

fn normalize_generation_variant(
    generation: &str,
    version_group: &str,
    shiny: bool,
    female: bool,
    animated: bool,
) -> (bool, bool, bool) {
    let supports_female = matches!(
        generation,
        "generation-iv"
            | "generation-v"
            | "generation-vi"
            | "generation-vii"
            | "generation-viii"
            | "generation-ix"
    );
    let supports_shiny = !matches!(generation, "generation-i" | "generation-ix");
    let supports_animated = (generation == "generation-ii" && version_group == "crystal")
        || (generation == "generation-v" && version_group == "black-white");

    (
        shiny && supports_shiny,
        female && supports_female,
        animated && supports_animated,
    )
}

fn default_version_group(generation: &str) -> &'static str {
    match generation {
        "generation-i" => "red-blue",
        "generation-ii" => "crystal",
        "generation-iii" => "emerald",
        "generation-iv" => "diamond-pearl",
        "generation-v" => "black-white",
        "generation-vi" => "x-y",
        "generation-vii" => "ultra-sun-ultra-moon",
        "generation-viii" => "icons",
        "generation-ix" => "scarlet-violet",
        _ => "red-blue",
    }
}

pub fn download_sprite(url: &str, cache_enabled: bool) -> Result<DynamicImage, String> {
    run_async(async {
        let client = build_http_client(cache_enabled)?;
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to download sprite '{}': {}", url, e))?;

        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("<unknown>")
            .to_string();

        if !status.is_success() {
            return Err(format!(
                "Failed to download sprite '{}': HTTP {} (content-type: {})",
                url, status, content_type
            ));
        }

        if !content_type.starts_with("image/") {
            return Err(format!(
                "Failed to download sprite '{}': expected image content, got '{}'",
                url, content_type
            ));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read sprite '{}': {}", url, e))?;

        decode_image_bytes(bytes.to_vec())
            .map_err(|e| format!("Failed to decode sprite '{}': {}", url, e))
    })
}

fn run_async<T, F>(future: F) -> Result<T, String>
where
    F: Future<Output = Result<T, String>>,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to initialize async runtime: {}", e))?;
    runtime.block_on(future)
}

fn build_rustemon_client(cache_enabled: bool) -> Result<rustemon::client::RustemonClient, String> {
    let manager = RustemonCACacheManager::new(cache_dir(), false);
    let mode = if cache_enabled {
        RustemonCacheMode::Default
    } else {
        RustemonCacheMode::NoStore
    };

    let builder: RustemonClientBuilder<RustemonCACacheManager> = RustemonClientBuilder::default();
    builder
        .with_manager(manager)
        .with_mode(mode)
        .try_build()
        .map_err(|e| format!("Failed to build rustemon client: {}", e))
}

fn build_http_client(cache_enabled: bool) -> Result<ClientWithMiddleware, String> {
    let mode = if cache_enabled {
        HttpCacheMode::Default
    } else {
        HttpCacheMode::NoStore
    };

    let reqwest = Client::builder()
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    Ok(ClientBuilder::new(reqwest)
        .with(HttpCacheMiddleware(HttpCache {
            mode,
            manager: HttpCACacheManager::new(cache_dir(), false),
            options: HttpCacheOptions::default(),
        }))
        .build())
}

fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ferrisdex")
        .join("http-cache")
}

fn decode_image_bytes(bytes: Vec<u8>) -> Result<DynamicImage, String> {
    ImageReader::new(io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| format!("Failed to read image format: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))
}
