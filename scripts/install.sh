#!/usr/bin/env bash
# Builds and installs the CapsLock Indicator cosmic-panel applet for the
# current user, or removes it with --uninstall.
set -euo pipefail

APP_ID="com.example.CosmicAppletCapsLock"
BIN_NAME="cosmic-applet-capslock"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
PROJECT_DIR="$(dirname -- "$SCRIPT_DIR")"

BIN_DEST_DIR="${HOME}/.local/bin"
DESKTOP_DEST_DIR="${HOME}/.local/share/applications"
DESKTOP_SRC="${PROJECT_DIR}/data/${APP_ID}.desktop"
DESKTOP_DEST="${DESKTOP_DEST_DIR}/${APP_ID}.desktop"
BIN_DEST="${BIN_DEST_DIR}/${BIN_NAME}"

uninstall() {
    echo "Removing ${BIN_DEST}"
    rm -f "$BIN_DEST"
    echo "Removing ${DESKTOP_DEST}"
    rm -f "$DESKTOP_DEST"
    echo "Done. You may need to restart cosmic-panel (or log out/in) for it to disappear."
}

if [[ "${1:-}" == "--uninstall" ]]; then
    uninstall
    exit 0
fi

echo "Building release binary..."
(cd "$PROJECT_DIR" && cargo build --release)

mkdir -p "$BIN_DEST_DIR" "$DESKTOP_DEST_DIR"

echo "Installing binary to ${BIN_DEST}"
install -m 755 "${PROJECT_DIR}/target/release/${BIN_NAME}" "$BIN_DEST"

echo "Installing desktop entry to ${DESKTOP_DEST}"
install -m 644 "$DESKTOP_SRC" "$DESKTOP_DEST"

case ":${PATH}:" in
    *":${BIN_DEST_DIR}:"*) ;;
    *)
        echo
        echo "WARNING: ${BIN_DEST_DIR} is not on your PATH."
        echo "cosmic-panel launches applets by the bare binary name on PATH, so add it,"
        echo "e.g. in ~/.profile or ~/.bashrc:"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        ;;
esac

echo
echo "Installed. Add it to a panel zone via:"
echo "  COSMIC Settings -> Desktop -> Panel -> (Start/Center/End) -> Add applet -> CapsLock Indicator"
echo "Run '${0} --uninstall' to remove it."
