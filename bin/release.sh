#!/bin/bash

ARCH="macos-$(uname -m)"
REPO="joacoc/psqlx"
TAP_REPO="joacoc/homebrew-psqlx"
VERSION="0.1.0"
BIN_NAME="psqlx"
BUILD_DIR="usr/local/psqlx/bin"
ARCHIVE_NAME="${BIN_NAME}-${ARCH}.tar.gz"
FORMULA_FILE="Formula/${BIN_NAME}.rb"

echo "🔨 Building ${BIN_NAME}..."
make install -C external/psql/src/bin/psql

echo "📦 Packaging binary..."
mkdir -p "$BUILD_DIR"
cp "/usr/local/pgsql/bin/${BIN_NAME}" "$BUILD_DIR"
tar -czvf "$ARCHIVE_NAME" -C "$BUILD_DIR" .

echo "📝 Calculating SHA256..."
SHA256=$(shasum -a 256 "$ARCHIVE_NAME" | awk '{print $1}')

echo "🚀 Uploading to GitHub Releases..."
gh release create "v$VERSION" "$ARCHIVE_NAME" --repo "$REPO" --notes "Release $VERSION"

echo "🌿 Cloning Homebrew tap..."
git clone "https://github.com/$TAP_REPO.git" homebrew-tap
cd homebrew-tap

echo "✏️ Updating Formula..."
cat <<EOF > "$FORMULA_FILE"
class Psqlx < Formula
  desc "Psql-fork focused on extensibility"
  homepage "https://github.com/$REPO"
  url "https://github.com/$REPO/releases/download/v$VERSION/$ARCHIVE_NAME"
  sha256 "$SHA256"
  version "$VERSION"

  depends_on "libpq"
  depends_on "libedit"

  def install
    bin.install "$BIN_NAME"
    libpq = Formula["libpq"].opt_lib
    libedit = Formula["libedit"].opt_lib
    system "install_name_tool", "-change", "/usr/local/pgsql/lib/libpq.5.dylib", "\#{libpq}/libpq.5.dylib", bin/"$BIN_NAME"
    system "install_name_tool", "-change", "/usr/lib/libedit.3.dylib", "\#{libedit}/libedit.3.dylib", bin/"$BIN_NAME"
  end

  test do
    system "\#{bin}/$BIN_NAME", "--version"
  end
end
EOF

echo "📤 Pushing Formula update..."
git add "$FORMULA_FILE"
git commit -m "Update $BIN_NAME to v$VERSION"
git push origin main

echo "✅ Done! Users can now run:"
echo "    brew update && brew install $BIN_NAME"
