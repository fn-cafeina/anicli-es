// Extractores de video: resuelven la URL directa del stream desde el embed
// de cada servidor. Sin dependencias externas: HTTP con ureq y búsqueda de
// patrones sobre el HTML crudo, sin motor JS.

mod mp4upload;

use super::provider::Server;
use anyhow::{bail, Result};

pub struct Extracted {
    pub url: String,
    pub referer: String,
}

pub fn extract(server: &Server) -> Result<Extracted> {
    match server.name.as_str() {
        "mp4upload" => mp4upload::extract(&server.url),
        name => bail!("servidor \"{name}\" sin extractor propio todavía"),
    }
}

// El embed muerto devuelve su misma página en vez del video; eso no se puede
// reproducir, así que se rechaza antes de lanzar el reproductor
pub fn is_stream_url(extracted: &str, embed_url: &str) -> bool {
    let extracted = extracted.trim().trim_end_matches('/');
    let embed_url = embed_url.trim().trim_end_matches('/');

    !extracted.is_empty() && !extracted.eq_ignore_ascii_case(embed_url)
}

// Los servidores exigen que el stream se pida desde su propio dominio:
// https://www.mp4upload.com/embed-abc.html -> https://www.mp4upload.com
pub fn referer_of(embed_url: &str) -> String {
    let Some(scheme_end) = embed_url.find("://") else {
        return String::new();
    };
    let after_scheme = &embed_url[scheme_end + 3..];
    let host_end = after_scheme.find('/').unwrap_or(after_scheme.len());
    embed_url[..scheme_end + 3 + host_end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server(name: &str) -> Server {
        Server {
            name: name.to_string(),
            url: "https://example.com/e/1".to_string(),
        }
    }

    #[test]
    fn rechaza_servidor_sin_extractor() {
        assert!(extract(&server("voe")).is_err());
    }

    #[test]
    fn referer_extrae_origen_del_embed() {
        assert_eq!(
            referer_of("https://www.mp4upload.com/embed-abc.html"),
            "https://www.mp4upload.com"
        );
        assert_eq!(referer_of("http://ok.ru/videoembed/123"), "http://ok.ru");
        assert_eq!(referer_of("https://host"), "https://host");
    }

    #[test]
    fn referer_vacio_sin_esquema() {
        assert_eq!(referer_of("no-es-una-url"), "");
    }

    #[test]
    fn extraccion_igual_al_embed_no_es_stream() {
        let embed = "https://www.mp4upload.com/embed-abc.html";

        assert!(!is_stream_url(embed, embed));
        assert!(!is_stream_url(
            "https://www.mp4upload.com/embed-abc.html/",
            embed
        ));
        assert!(!is_stream_url(
            "HTTPS://WWW.MP4UPLOAD.COM/EMBED-ABC.HTML",
            embed
        ));
        assert!(!is_stream_url("", embed));
        assert!(!is_stream_url("   ", embed));
    }

    #[test]
    fn url_distinta_al_embed_es_stream() {
        assert!(is_stream_url(
            "https://a4.mp4upload.com:183/d/xyz/video.mp4",
            "https://www.mp4upload.com/embed-abc.html"
        ));
    }
}
