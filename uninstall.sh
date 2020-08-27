#!/usr/bin/env bash

if [ -z "$PREFIX" ]; then
  PREFIX=/usr/local
fi

sudo rm -v $PREFIX/bin/xyz.gelez.mobydick
sudo rm -v $PREFIX/share/appdata/$(ls *.appdata.xml)
sudo rm -v $PREFIX/share/applications/$(ls *.desktop)

for s in "16" "24" "32" "48" "64" "128"; do
  sudo rm -v $PREFIX/share/icons/hicolor/${s}x${s}/mobydick.svg
done
