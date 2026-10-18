extern crate getopts;
use anyhow::Result;
use dialoguer::Input;
use getopts::Options;
use std::{env, process};

mod extractor;
mod http;
mod player;
mod provider;
mod ui;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let project_name = option_env!("PROJECT_NAME").unwrap_or("anicli-es");

    let mut options = Options::new();
    options.optflag("h", "help", "obtener ayuda para los comandos");
    options.optflag("", "version", "mostrar la versión");

    let matches = options.parse(&args[1..])?;

    if matches.opt_present("h") {
        let brief = format!("Uso: {} [busqueda] [opciones]", project_name);
        print!("{}", options.usage(&brief));
        return Ok(());
    }

    if matches.opt_present("version") {
        println!("{} {}", project_name, env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let base = provider::base_url();
    let mut pending_query = if matches.free.is_empty() {
        None
    } else {
        Some(matches.free.join(" "))
    };

    'search: loop {
        let query = match pending_query.take() {
            Some(query) => query,
            None => Input::new().with_prompt("Buscar animé").interact()?,
        };

        let resultados = match provider::search(&base, &query) {
            Ok(resultados) => resultados,
            Err(error) => {
                println!("No se pudo buscar: {error:#}");
                continue;
            }
        };

        if resultados.is_empty() {
            println!("No se encontró ningun animé con ese nombre.");
            continue;
        }

        let seleccion = ui::choose_anime(&resultados)?;
        let anime = &resultados[seleccion];
        println!("Elegiste {}", anime.title);

        let episodios = match provider::episodes(&anime.url) {
            Ok(episodios) => episodios,
            Err(error) => {
                println!("No se pudieron obtener los episodios: {error:#}");
                continue;
            }
        };

        if episodios.is_empty() {
            println!("No se encontrarón episodios, intenta con otro animé.");
            continue;
        }

        let mut index = ui::choose_index(
            episodios.len(),
            &format!("episodio [1-{}]", episodios.len()),
        )?;

        loop {
            match player::play(anime, &episodios, index)? {
                ui::Action::Replay => {}
                ui::Action::Switch(next) => index = next,
                ui::Action::SearchAnother => continue 'search,
                ui::Action::Exit => process::exit(0),
            }
        }
    }
}
