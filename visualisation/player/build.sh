#!/bin/sh
# Records every scenario from fresh DAGLight runs into visualisation/player/replays.js;
# pass --open to open the player afterwards.
set -e
cd "$(dirname "$0")/../.."
cargo run --release --quiet -p daglight-visualisation-recorder -- visualisation/player/replays.js
if [ "$1" = "--open" ]; then
  xdg-open visualisation/player/index.html 2>/dev/null || open visualisation/player/index.html
else
  echo "open visualisation/player/index.html in a browser"
fi
