set -e
PLAT=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
if [ "$PLAT" = "darwin" ] && [ "$ARCH" = "arm64" ]; then TARGET=aarch64-apple-darwin; fi
if [ "$PLAT" = "darwin" ] && [ "$ARCH" != "arm64" ]; then TARGET=x86_64-apple-darwin; fi
if [ "$PLAT" = "linux" ]; then TARGET=x86_64-unknown-linux-gnu; fi
ASSET=kandil-$TARGET.tar.gz
URL=https://github.com/Kandil7/kandil_code/releases/latest/download/$ASSET
SHAURL=https://github.com/Kandil7/kandil_code/releases/latest/download/kandil-$TARGET.sha256
TMP=$(mktemp -d)
cd "$TMP"
curl -fsSL "$URL" -o "$ASSET"
curl -fsSL "$SHAURL" -o SHA256SUM
HASH_EXPECTED=$(cat SHA256SUM | tr -d '\r\n')
HASH_ACTUAL=$(sha256sum "$ASSET" | cut -d ' ' -f1 2>/dev/null || shasum -a 256 "$ASSET" | cut -d ' ' -f1)
if [ "$HASH_EXPECTED" != "$HASH_ACTUAL" ]; then echo "checksum mismatch"; exit 1; fi
mkdir -p kandil
tar xzf "$ASSET" -C kandil
DEST="$HOME/.local/bin"
mkdir -p "$DEST"
install -m 755 kandil/kandil "$DEST/kandil"
echo "$DEST" | grep -q "$HOME/.local/bin" || true
echo "$DEST" >/dev/null
echo "installed to $DEST"
