# anicli-es

Herramienta para ver animé desde la línea de comandos, con subtitulos en español; usa el sitio [monoschinos](https://monoschinos.st/) para extraer los animes.

## Uso

	# Ver animé
	anicli-es <query>

<p align="center">
  <img src="./anicli-es.gif" alt="usage" />
</p>
	
## Instalación

### Binario

Descarga `anicli-es` desde [releases](https://github.com/fn-cafeina/anicli-es/releases), descomprime el `.tar.gz` y mueve el binario a un directorio en tu `$PATH`.

### Cargo
	
Instala cargo con el administrador de paquetes de tu distribución de Linux y ejecuta:
`$ cargo install --git https://github.com/fn-cafeina/anicli-es`
	
Asegurate de que donde se instale el binario, usualmente `$HOME/.cargo/bin` sea parte de $PATH para poder ejecutarlo donde sea.

## Dependencias

* mpv
* yt-dlp

yt-dlp extrae la URL del video desde el servidor embebido; mpv lo reproduce. Por ahora solo se soporta el servidor **mp4upload**.

## Notas

En WSL (WSLg) mpv usa render por software automáticamente; no hay que configurar nada.

## Configuración

El sitio se extrae de `https://monoschinos.st` por defecto. Si cambia de dominio:

	ANICLI_ES_BASE_URL=https://otro-dominio anicli-es <query>

## Desarrollo

Los tests usan fixtures reales del sitio, que no se versionan:

	scripts/fetch_fixtures.sh
	cargo test
