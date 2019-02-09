cargo build --release
sudo cp target/release/mobydick $PREFIX/bin/xyz.gelez.mobydick
sudo cp *.appdata.xml $PREFIX/share/appdata/
sudo cp *.desktop $PREFIX/share/applications/
for s in "16" "24" "32" "48" "64" "128"; do
  sudo cp icons/$s.svg $PREFIX/share/icons/hicolor/${s}x${s}/mobydick.svg
done