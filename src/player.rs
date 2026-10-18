use crate::extractor;
use crate::provider::{self, Anime};
use crate::ui;
use anyhow::{Context, Result};
use std::process::Command;

// Elige servidor, extrae el video y lo reproduce en mpv
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
    let extracted = match extractor::extract(server) {
        Ok(extracted) => extracted,
        Err(error) => {
            println!("No se pudo extraer el video: {error:#}");
            return pick_another_episode(episodes);
        }
    };

    if let Err(error) = launch_mpv(&extracted.url, &extracted.referer) {
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
}
