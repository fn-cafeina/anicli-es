use anyhow::{Context, Result};
use std::time::Duration;

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:140.0) Gecko/20100101 Firefox/140.0";

// Devuelve el código fuente de una página web como String
pub fn get(url: &str) -> Result<String> {
    let response = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .timeout(Duration::from_secs(20))
        .call()
        .with_context(|| format!("no se pudo conectar a {url}"))?;

    response
        .into_string()
        .with_context(|| format!("no se pudo leer la respuesta de {url}"))
}
