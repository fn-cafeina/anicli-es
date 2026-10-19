// mp4upload sirve su embed como HTML plano con videojs: la URL directa viene
// literal en `player.src({ src: "https://a3.mp4upload.com:183/d/.../video.mp4" })`.
// Sin JS ofuscado ni eval packing: basta buscar el substring `src: "..."`.
use super::{is_stream_url, referer_of, Extracted};
use crate::http;
use anyhow::{bail, Context, Result};

// Prefijos literales que anteceden a la URL del stream, en orden de preferencia
const SRC_MARKERS: &[&str] = &["src:", "file:"];

pub fn extract(embed_url: &str) -> Result<Extracted> {
    let html =
        http::get(embed_url).with_context(|| format!("descargando el embed de {embed_url}"))?;
    let url = find_stream_url(&html, embed_url)
        .with_context(|| format!("sin stream en {embed_url} (link caído o markup nuevo)"))?;
    Ok(Extracted {
        url,
        referer: referer_of(embed_url),
    })
}

// Primera URL http(s) tras un marcador `src:`/`file:` que no sea el propio embed
fn find_stream_url(html: &str, embed_url: &str) -> Result<String> {
    for marker in SRC_MARKERS {
        let mut rest = html;
        while let Some(pos) = rest.find(marker) {
            rest = &rest[pos + marker.len()..];
            let after = rest.trim_start_matches([' ', '\t', '\n', '\r']);
            if let Some(url) = quoted_url(after) {
                if (url.starts_with("http://") || url.starts_with("https://"))
                    && is_stream_url(&url, embed_url)
                {
                    return Ok(url);
                }
            }
        }
    }
    bail!("no se encontró src del video en el embed");
}

// URL entre comillas simples o dobles al inicio del texto
fn quoted_url(text: &str) -> Option<String> {
    let quote = text.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    Some(
        text[quote.len_utf8()..]
            .chars()
            .take_while(|c| *c != quote)
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider;

    #[test]
    fn extrae_url_directa_del_fixture() {
        let html = provider::fixture("mp4upload_embed.html");
        let embed = "https://www.mp4upload.com/embed-gkqwh8owbh12.html";

        let url = find_stream_url(&html, embed).expect("el fixture trae el src");

        assert!(
            url.starts_with("https://") && url.ends_with("/video.mp4"),
            "stream mp4 directo, fue: {url}"
        );
        assert!(url.contains("mp4upload.com"), "mismo CDN, fue: {url}");
        assert!(is_stream_url(&url, embed));
    }

    #[test]
    fn embed_muerto_da_error() {
        let embed = "https://www.mp4upload.com/embed-abc.html";

        assert!(find_stream_url("File was deleted", embed).is_err());
        assert!(find_stream_url("", embed).is_err());
        assert!(find_stream_url(&format!(r#"src: "{embed}""#), embed).is_err());
    }

    #[test]
    fn acepta_comillas_simples_y_file() {
        let embed = "https://www.mp4upload.com/embed-abc.html";
        let stream = "https://a3.mp4upload.com:183/d/xyz/video.mp4";

        assert_eq!(
            find_stream_url(&format!("src: '{stream}'"), embed).unwrap(),
            stream
        );
        assert_eq!(
            find_stream_url(&format!(r#"file: "{stream}""#), embed).unwrap(),
            stream
        );
    }
}
