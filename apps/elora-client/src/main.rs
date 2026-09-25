//! Elora-Client. In M1 ist er die Physik-Sandbox (E-013).

fn main() {
    println!(
        "Elora {} – {} Ticks/s, Tile-Größe {}",
        env!("CARGO_PKG_VERSION"),
        elora_sim::TICKS_PER_SECOND,
        elora_map::TILE_SIZE,
    );
}
