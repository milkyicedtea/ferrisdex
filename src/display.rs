use rustemon::model::pokemon::Pokemon;

pub fn build_info_lines(pokemon: &Pokemon, display_name: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    lines.push(String::new());
    lines.push("🦀 FerrisDex".to_string());
    lines.push("══════════════════════════".to_string());
    lines.push(String::new());
    lines.push(format!("  Name:  {}", display_name));
    lines.push(format!("  Dex #: {}", pokemon.id));
    lines.push(String::new());
    lines.push("  ── Abilities ──".to_string());
    for ability in &pokemon.abilities {
        if let Some(ability_ref) = &ability.ability {
            lines.push(format!("    • {}", ability_ref.name));
        }
    }
    lines.push(String::new());
    lines.push("  ── Base Stats ──".to_string());
    for stat in &pokemon.stats {
        let bar_len = (stat.base_stat / 5).min(20) as usize;
        let bar = "█".repeat(bar_len);
        lines.push(format!(
            "    {:<20} {:>3}  {}",
            stat.stat.name, stat.base_stat, bar
        ));
    }
    lines.push(String::new());

    lines
}
