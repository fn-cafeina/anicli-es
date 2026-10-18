use super::{Anime, Server};
use crate::http;
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use scraper::{Html, Selector};

// Busca animes en el sitio y devuelve los resultados de la primera página
pub fn search(base: &str, query: &str) -> Result<Vec<Anime>> {
    let base = base.trim_end_matches('/');
    let url = format!("{base}/buscar?q={}", urlencoding::encode(query.trim()));
    let html = http::get(&url)?;
    parse_search(&html, base).with_context(|| format!("parseando los resultados de {url}"))
}

// Devuelve las URLs de todos los episodios de un anime, en orden
pub fn episodes(anime_url: &str) -> Result<Vec<String>> {
    let html = http::get(anime_url)?;
    let (ajax_url, ver_base) =
        parse_anime_page(&html).with_context(|| format!("parseando la página de {anime_url}"))?;

    let json = http::get(&ajax_url)?;
    let nums = parse_episode_nums(&json)
        .with_context(|| format!("parseando los episodios de {ajax_url}"))?;

    Ok(nums
        .into_iter()
        .map(|num| format!("{ver_base}{num}"))
        .collect())
}

// Devuelve los servidores disponibles de un episodio
pub fn servers(episode_url: &str) -> Result<Vec<Server>> {
    let html = http::get(episode_url)?;
    parse_servers(&html).with_context(|| format!("parseando los servidores de {episode_url}"))
}

fn parse_search(html: &str, base: &str) -> Result<Vec<Anime>> {
    let doc = Html::parse_document(html);
    let card_sel = Selector::parse("a.card-wrap").expect("selector válido");
    let title_sel = Selector::parse("h3.card-title").expect("selector válido");
    let info_sel = Selector::parse("div.mt-1").expect("selector válido");

    let mut animes = Vec::new();
    for card in doc.select(&card_sel) {
        let url = match card.value().attr("href") {
            Some(href) if href.starts_with("http") => href.to_string(),
            Some(href) => format!("{}{}", base, href),
            None => continue,
        };

        let title = card
            .select(&title_sel)
            .next()
            .map(|el| normalize_text(&el))
            .unwrap_or_default();
        if title.is_empty() {
            continue;
        }

        let info = card
            .select(&info_sel)
            .next()
            .map(|el| normalize_text(&el))
            .unwrap_or_default();

        animes.push(Anime { title, info, url });
    }

    Ok(animes)
}

// De la página del anime saca la URL del endpoint de episodios y la base
// de los links "/ver/" (ej: https://monoschinos.st/ver/naruto-episodio-)
fn parse_anime_page(html: &str) -> Result<(String, String)> {
    let doc = Html::parse_document(html);

    let ajax_sel = Selector::parse("[data-ajax]").expect("selector válido");
    let ajax_url = doc
        .select(&ajax_sel)
        .next()
        .and_then(|el| el.value().attr("data-ajax"))
        .map(str::to_string);

    let ver_sel = Selector::parse("a[href*='/ver/']").expect("selector válido");
    let ver_links: Vec<String> = doc
        .select(&ver_sel)
        .filter_map(|el| el.value().attr("href"))
        .map(str::to_string)
        .collect();

    match (ajax_url, ver_links.into_iter().next()) {
        (Some(ajax), Some(ver)) => Ok((ajax, ver_base_of(&ver)?)),
        _ => Err(anyhow!(
            "no se encontró la lista de episodios; puede que el sitio haya cambiado"
        )),
    }
}

// https://monoschinos.st/ver/naruto-episodio-1 -> https://monoschinos.st/ver/naruto-episodio-
fn ver_base_of(ver_url: &str) -> Result<String> {
    let base = ver_url.trim_end_matches(|c: char| c.is_ascii_digit());
    if base.len() == ver_url.len() {
        return Err(anyhow!("link de episodio inesperado: {ver_url}"));
    }
    Ok(base.to_string())
}

fn parse_episode_nums(json: &str) -> Result<Vec<u32>> {
    #[derive(serde::Deserialize)]
    struct Pagination {
        eps: Vec<Episode>,
    }
    #[derive(serde::Deserialize)]
    struct Episode {
        num: u32,
    }

    let pagination: Pagination = serde_json::from_str(json)?;
    Ok(pagination.eps.into_iter().map(|ep| ep.num).collect())
}

// Los servidores vienen en botones con el embed en base64: data-player="aHR0cHM6..."
fn parse_servers(html: &str) -> Result<Vec<Server>> {
    let doc = Html::parse_document(html);
    let player_sel = Selector::parse("[data-player]").expect("selector válido");

    let mut servers = Vec::new();
    for el in doc.select(&player_sel) {
        let Some(encoded) = el.value().attr("data-player") else {
            continue;
        };
        let Ok(bytes) = STANDARD.decode(encoded) else {
            continue;
        };
        let Ok(url) = String::from_utf8(bytes) else {
            continue;
        };

        servers.push(Server {
            name: normalize_text(&el),
            url,
        });
    }

    if servers.is_empty() {
        return Err(anyhow!(
            "no se encontró ningún servidor; puede que el sitio haya cambiado"
        ));
    }

    Ok(servers)
}

fn normalize_text(el: &scraper::ElementRef) -> String {
    el.text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://monoschinos.st";

    #[test]
    fn busqueda_extrae_titulos_info_y_links() {
        let animes = parse_search(&crate::provider::fixture("buscar_naruto.html"), BASE).unwrap();

        assert!(animes.len() >= 10);
        assert!(animes.iter().any(|anime| anime.title == "Naruto"));
        assert!(animes.iter().all(|anime| anime.url.starts_with("https://")));
        assert!(animes.iter().all(|anime| !anime.info.is_empty()));
    }

    #[test]
    fn busqueda_sin_resultados_devuelve_vacio() {
        let animes = parse_search(&crate::provider::fixture("buscar_vacio.html"), BASE).unwrap();
        assert!(animes.is_empty());
    }

    #[test]
    fn pagina_de_anime_entrega_ajax_y_base_de_ver() {
        let (ajax_url, ver_base) =
            parse_anime_page(&crate::provider::fixture("anime_naruto.html")).unwrap();

        assert_eq!(ajax_url, "https://monoschinos.st/ajax/ajax_pagination/278");
        assert_eq!(ver_base, "https://monoschinos.st/ver/naruto-episodio-");
    }

    #[test]
    fn pagina_de_pelicula_tambien_entrega_ajax_y_base_de_ver() {
        let (ajax_url, ver_base) =
            parse_anime_page(&crate::provider::fixture("anime_movie.html")).unwrap();

        assert_eq!(ajax_url, "https://monoschinos.st/ajax/ajax_pagination/77");
        assert_eq!(
            ver_base,
            "https://monoschinos.st/ver/baruto-the-movie-episodio-"
        );
    }

    #[test]
    fn episodios_lista_completa_en_orden() {
        let nums = parse_episode_nums(&crate::provider::fixture("episodios_278.json")).unwrap();

        assert_eq!(nums.len(), 220);
        assert_eq!(nums.first(), Some(&1));
        assert_eq!(nums.last(), Some(&220));
    }

    #[test]
    fn servidores_decodifica_base64_con_nombre() {
        let servers = parse_servers(&crate::provider::fixture("ver_naruto_ep1.html")).unwrap();

        assert_eq!(servers.len(), 8);
        let doodstream = servers
            .iter()
            .find(|server| server.name == "doodstream")
            .unwrap();
        assert_eq!(doodstream.url, "https://doodstream.com/e/7rzty4dot1pe");
        assert!(servers.iter().all(|server| server.url.starts_with("http")));
    }

    #[test]
    fn ver_base_rechaza_link_sin_numero() {
        assert!(ver_base_of("https://monoschinos.st/ver/naruto-episodio-a").is_err());
    }

    // Prueba de humo contra el sitio real; corre con: cargo test -- --ignored
    #[test]
    #[ignore]
    fn live_smoke_sitio_real() {
        let base = "https://monoschinos.st";

        let animes = search(base, "naruto").unwrap();
        assert!(!animes.is_empty());

        let naruto = animes
            .iter()
            .find(|anime| anime.title == "Naruto")
            .expect("Naruto debe aparecer en los resultados");

        let episodios = episodes(&naruto.url).unwrap();
        assert_eq!(episodios.len(), 220);
        assert_eq!(episodios[0], "https://monoschinos.st/ver/naruto-episodio-1");

        let servers = servers(&episodios[0]).unwrap();
        assert!(!servers.is_empty());
    }
}
