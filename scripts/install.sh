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
# cosmic-panel resolves Exec= against its own session PATH, which (unlike an
# interactive shell's PATH) generally does NOT include ~/.local/bin -- so a
# bare `Exec=cosmic-applet-capslock` silently fails to launch. Substitute the
# absolute path so the applet starts regardless of the session's PATH.
sed "s|^Exec=${BIN_NAME}\$|Exec=${BIN_DEST}|" "$DESKTOP_SRC" > "$DESKTOP_DEST"
chmod 644 "$DESKTOP_DEST"

echo
echo "Installed. Add it to a panel zone via:"
echo "  COSMIC Settings -> Desktop -> Panel -> (Start/Center/End) -> Add applet -> CapsLock Indicator"
echo "Run '${0} --uninstall' to remove it."
