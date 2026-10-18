use crate::provider::{Anime, Server};
use anyhow::Result;
use dialoguer::Input;

pub enum Action {
    Replay,
    Switch(usize),
    SearchAnother,
    Exit,
}

// Pide un índice en el rango [1, length] y devuelve el índice base 0
pub fn choose_index(length: usize, what: &str) -> Result<usize> {
    if length == 1 {
        return Ok(0);
    }

    loop {
        let input: String = Input::new()
            .with_prompt(format!("Elige un {what}"))
            .interact()?;

        match input.trim().parse::<usize>() {
            Ok(index) if index >= 1 && index <= length => return Ok(index - 1),
            Ok(_) => println!("El índice seleccionado es invalido."),
            Err(_) => println!("Porfavor escribe un número."),
        }
    }
}

pub fn choose_anime(animes: &[Anime]) -> Result<usize> {
    if animes.len() == 1 {
        return Ok(0);
    }

    for (n, anime) in animes.iter().enumerate() {
        if anime.info.is_empty() {
            println!("[{}] {}", n + 1, anime.title);
        } else {
            println!("[{}] {} - {}", n + 1, anime.title, anime.info);
        }
    }

    choose_index(animes.len(), &format!("animé [1-{}]", animes.len()))
}

pub fn choose_server(servers: &[Server]) -> Result<usize> {
    if servers.len() == 1 {
        return Ok(0);
    }

    for (n, server) in servers.iter().enumerate() {
        println!("[{}] {}", n + 1, server.name);
    }

    choose_index(servers.len(), &format!("servidor [1-{}]", servers.len()))
}

// Menú posterior a la reproducción; decide el próximo paso
pub fn controller(index: usize, total: usize, anime: &str) -> Result<Action> {
    let has_prev = index > 0;
    let has_next = index + 1 < total;

    loop {
        let mut prompt = String::new();
        if has_next {
            prompt.push_str("[s] Siguiente episodio\n");
        }
        if has_prev {
            prompt.push_str("[a] Anterior episodio\n");
        }
        prompt.push_str(
            "[r] Ver de nuevo\n[o] Seleccionar otro episodio\n[b] Buscar otro anime\n[q] Salir\nEscoge una opción",
        );

        let option: String = Input::new().with_prompt(prompt).interact()?;

        match option.trim().to_lowercase().as_str() {
            "q" => return Ok(Action::Exit),
            "b" => return Ok(Action::SearchAnother),
            "r" => return Ok(Action::Replay),
            "o" => {
                return Ok(Action::Switch(choose_index(
                    total,
                    &format!("episodio [1-{total}]"),
                )?))
            }
            "s" if has_next => return Ok(Action::Switch(index + 1)),
            "a" if has_prev => return Ok(Action::Switch(index - 1)),
            _ => {
                println!("Escoge una opción valida.\n");
                println!("Viendo \"{anime}\", episodio {}.", index + 1);
            }
        }
    }
}
