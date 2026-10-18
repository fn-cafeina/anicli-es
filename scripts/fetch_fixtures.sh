#!/usr/bin/env bash
# Descarga HTML/JSON reales del sitio para los tests offline.
# Uso: scripts/fetch_fixtures.sh
set -euo pipefail

cd "$(dirname "$0")/.."
mkdir -p tests/fixtures

BASE="${ANICLI_ES_BASE_URL:-https://monoschinos.st}"
UA="Mozilla/5.0 (X11; Linux x86_64; rv:140.0) Gecko/20100101 Firefox/140.0"

echo "Descargando fixtures desde $BASE ..."

curl -fsSL -A "$UA" "$BASE/buscar?q=naruto" -o tests/fixtures/buscar_naruto.html
curl -fsSL -A "$UA" "$BASE/buscar?q=zzzxxyyqq123" -o tests/fixtures/buscar_vacio.html
curl -fsSL -A "$UA" "$BASE/anime/naruto-sub-espanol" -o tests/fixtures/anime_naruto.html
curl -fsSL -A "$UA" "$BASE/anime/baruto-the-movie-sub-espanol" -o tests/fixtures/anime_movie.html
curl -fsSL -A "$UA" -H "X-Requested-With: XMLHttpRequest" \
    "$BASE/ajax/ajax_pagination/278" -o tests/fixtures/episodios_278.json
curl -fsSL -A "$UA" "$BASE/ver/naruto-episodio-1" -o tests/fixtures/ver_naruto_ep1.html

echo "Listo: $(ls tests/fixtures | wc -l) fixtures en tests/fixtures/"
