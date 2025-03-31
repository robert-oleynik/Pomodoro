#!/bin/sh

set -xe

DATADIR="$HOME/.local/share"

cp "local.app.Pomodoro.gschema.xml" "$DATADIR/glib-2.0/schemas"
glib-compile-schemas "$HOME/.local/share/glib-2.0/schemas"

cp "Pomodoro.desktop" "$DATADIR/applications"

cargo install --path .
