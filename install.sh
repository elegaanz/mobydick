#!/usr/bin/env sh
if [ -z "$PREFIX" ]; then
  PREFIX=/usr/local
fi

if [ ! -f target/release/mobydick ]; then
  ./build.sh
fi

sudo install -v -d $PREFIX/bin/
sudo install -v target/release/mobydick $PREFIX/bin/xyz.gelez.mobydick
sudo install -v -d $PREFIX/share/appdata
sudo install -v -t $PREFIX/share/appdata *.appdata.xml
sudo install -v -d $PREFIX/share/applications
sudo install -v -t $PREFIX/share/applications *.desktop
for s in "16" "24" "32" "48" "64" "128"; do
  sudo install -v -d $PREFIX/share/icons/hicolor/${s}x${s}/ 
  sudo install -v icons/$s.svg $PREFIX/share/icons/hicolor/${s}x${s}/mobydick.svg
done
