use crate::provider::{self, Anime};
use crate::ui;
use anyhow::{bail, Context, Result};
use std::process::Command;

// Elige servidor, extrae el video con yt-dlp y lo reproduce en mpv
pub fn play(anime: &Anime, episodes: &[String], index: usize) -> Result<ui::Action> {
    println!(
        "{esc}[2J{esc}[1;1HObteniendo información del video.\n",
        esc = 27 as char
    );

    let servers = match provider::servers(&episodes[index]) {
        Ok(servers) if !servers.is_empty() => servers,
        Ok(_) => {
            println!("No se encontró ningun servidor útil.");
            return pick_another_episode(episodes);
        }
        Err(error) => {
            println!("No se pudieron obtener los servidores: {error:#}");
            return pick_another_episode(episodes);
        }
    };

    let servers = supported_servers(servers);
    if servers.is_empty() {
        println!(
            "Este episodio no tiene servidores soportados (por ahora: {}).",
            provider::SUPPORTED_SERVERS.join(", ")
        );
        return pick_another_episode(episodes);
    }

    let server = &servers[ui::choose_server(&servers)?];

    println!("Extrayendo video desde \"{}\"...\n", server.name);
    let url = match extract_direct_url(&server.url) {
        Ok(url) => url,
        Err(error) => {
            println!("No se pudo extraer el video: {error:#}");
            return pick_another_episode(episodes);
        }
    };

    if let Err(error) = launch_mpv(&url, &referer_of(&server.url)) {
        println!("{error:#}");
        return pick_another_episode(episodes);
    }

    if episodes.len() == 1 {
        println!(
            "Viendo \"{}\" desde el servidor \"{}\".\n",
            anime.title, server.name
        );
    } else {
        println!(
            "Viendo \"{}\", episodio {} (servidor \"{}\").\n",
            anime.title,
            index + 1,
            server.name
        );
    }

    ui::controller(index, episodes.len(), &anime.title)
}

fn pick_another_episode(episodes: &[String]) -> Result<ui::Action> {
    Ok(ui::Action::Switch(ui::choose_index(
        episodes.len(),
        &format!("episodio [1-{}]", episodes.len()),
    )?))
}

fn supported_servers(servers: Vec<provider::Server>) -> Vec<provider::Server> {
    servers
        .into_iter()
        .filter(|server| provider::SUPPORTED_SERVERS.contains(&server.name.as_str()))
        .collect()
}

// Pide a yt-dlp la URL directa del stream del servidor embebido
fn extract_direct_url(embed_url: &str) -> Result<String> {
    let output = Command::new("yt-dlp")
        .args(["-g", "--no-warnings", embed_url])
        .output()
        .context("no se pudo ejecutar yt-dlp (¿está instalado?)")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let reason = stderr.lines().last().unwrap_or("").trim().to_string();
        bail!("yt-dlp no pudo extraer el video: {reason}");
    }

    let url = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();

    if !is_stream_url(&url, embed_url) {
        bail!("el servidor devolvió su página en vez del video (link caído); prueba otro episodio");
    }

    Ok(url)
}

// yt-dlp "extrae" la misma página del embed cuando el link está muerto; mpv
// no puede reproducir eso, así que se rechaza antes de lanzarlo
fn is_stream_url(extracted: &str, embed_url: &str) -> bool {
    let extracted = extracted.trim().trim_end_matches('/');
    let embed_url = embed_url.trim().trim_end_matches('/');

    !extracted.is_empty() && !extracted.eq_ignore_ascii_case(embed_url)
}

// Los servidores exigen que el stream se pida desde su propio dominio:
// https://www.mp4upload.com/embed-abc.html -> https://www.mp4upload.com
fn referer_of(embed_url: &str) -> String {
    let Some(scheme_end) = embed_url.find("://") else {
        return String::new();
    };
    let after_scheme = &embed_url[scheme_end + 3..];
    let host_end = after_scheme.find('/').unwrap_or(after_scheme.len());
    embed_url[..scheme_end + 3 + host_end].to_string()
}

// mpv reproduce la URL directa; setsid lo deja en segundo plano para que la
// terminal siga libre mientras se reproduce
fn launch_mpv(url: &str, referer: &str) -> Result<()> {
    let mut command = Command::new("setsid");
    command
        .arg("-f")
        .arg("mpv")
        .arg("--no-ytdl")
        .arg("--msg-level=all=warn")
        .arg(format!("--http-header-fields=Referer: {referer}"));

    // WSLg no expone la GPU a Linux: sin esto mpv muere intentando Vulkan.
    // En Linux nativo se dejan los valores por defecto de mpv
    if is_wsl() {
        command.env("LIBGL_ALWAYS_SOFTWARE", "1").args([
            "--vo=gpu",
            "--gpu-api=opengl",
            "--gpu-context=x11egl",
            "--msg-level=all=warn,vo/gpu/opengl=error",
        ]);
    }

    command
        .arg(url)
        .spawn()
        .context("no se pudo lanzar mpv (¿está instalado?)")?;

    Ok(())
}

fn is_wsl() -> bool {
    std::env::var_os("WSL_DISTRO_NAME").is_some()
        || std::fs::read_to_string("/proc/version")
            .map(|version| version.to_lowercase().contains("microsoft"))
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::Server;

    fn server(name: &str) -> Server {
        Server {
            name: name.to_string(),
            url: "https://example.com/e/1".to_string(),
        }
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
    fn filtro_deja_solo_servidores_soportados() {
        let servers = vec![server("voe"), server("mp4upload"), server("mixdrop")];

        let filtered = supported_servers(servers);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "mp4upload");
    }

    #[test]
    fn filtro_vacio_si_no_hay_soportados() {
        let servers = vec![server("voe")];

        assert!(supported_servers(servers).is_empty());
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
