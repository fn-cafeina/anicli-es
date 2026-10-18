// Proveedor de datos: sitios desde donde se extraen animes, episodios y servidores.

mod monoschinos;

pub use monoschinos::{episodes, search, servers};

pub const DEFAULT_BASE_URL: &str = "https://monoschinos.st";

// Servidores cuyos videos sabemos extraer; el resto (voe, uqload, doodstream,
// mixdrop, etc.) cambió sus mirrors y sus links viejos están muertos, así que
// se abordan más adelante
pub const SUPPORTED_SERVERS: &[&str] = &["mp4upload"];

// Dominio configurable porque el sitio ya cambió de casa más de una vez
pub fn base_url() -> String {
    std::env::var("ANICLI_ES_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
}

#[derive(Debug, Clone)]
pub struct Anime {
    pub title: String,
    pub info: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub name: String,
    pub url: String,
}
