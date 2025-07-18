#!/bin/sh

set -e

if [ -n "${DEBUG}" ]; then
  set -x
fi

# デフォルト設定
DEFAULT_INSTALL_PATH="/usr/local/bin"
WMGR_REPO="tk-aria/wmgr"

# 最新バージョンを取得
_wmgr_latest() {
  curl -sSLf "https://api.github.com/repos/${WMGR_REPO}/releases/latest" | \
    grep '"tag_name":' | \
    sed -E 's/.*"([^"]+)".*/\1/'
}

# OS検出
_detect_os() {
  os="$(uname -s)"
  case "$os" in
    Linux) echo "linux" ;;
    Darwin) echo "darwin" ;;
    CYGWIN*|MINGW*|MSYS*) echo "windows" ;;
    *) echo "Unsupported operating system: $os" 1>&2; return 1 ;;
  esac
  unset os
}

# アーキテクチャ検出
_detect_arch() {
  arch="$(uname -m)"
  case "$arch" in
    amd64|x86_64) echo "x86_64" ;;
    arm64|aarch64) echo "aarch64" ;;
    armv7l|armv8l|arm) echo "armv7" ;;
    *) echo "Unsupported processor architecture: $arch" 1>&2; return 1 ;;
  esac
  unset arch
}

# バイナリ名を決定
_get_binary_name() {
  os="$1"
  case "$os" in
    windows) echo "wmgr.exe" ;;
    *) echo "wmgr" ;;
  esac
}

# ダウンロードURL生成
_download_url() {
  local version="$1"
  local os="$2"
  local arch="$3"
  
  # バイナリファイル名: wmgr-{version}-{os}-{arch}.tar.gz
  local archive_name="wmgr-${version}-${os}-${arch}.tar.gz"
  echo "https://github.com/${WMGR_REPO}/releases/download/${version}/${archive_name}"
}

# インストール実行
main() {
  # バージョン決定
  if [ -z "${WMGR_VERSION}" ]; then
    echo "Getting latest version..."
    WMGR_VERSION=$(_wmgr_latest)
    if [ -z "${WMGR_VERSION}" ]; then
      echo "Failed to get latest version" 1>&2
      return 1
    fi
  fi

  # インストールパス決定
  wmgr_install_path="${WMGR_INSTALL_PATH:-$DEFAULT_INSTALL_PATH}"
  
  # プラットフォーム検出
  wmgr_os="$(_detect_os)"
  wmgr_arch="$(_detect_arch)"
  wmgr_binary="$(_get_binary_name "$wmgr_os")"
  
  # ダウンロードURL生成
  wmgr_download_url="$(_download_url "$WMGR_VERSION" "$wmgr_os" "$wmgr_arch")"

  echo "Installing wmgr ${WMGR_VERSION} for ${wmgr_os}/${wmgr_arch}..."
  echo "Download URL: $wmgr_download_url"

  # インストールディレクトリ作成
  if [ ! -d "$wmgr_install_path" ]; then
    echo "Creating install directory: $wmgr_install_path"
    mkdir -p -- "$wmgr_install_path"
  fi

  # 一時ディレクトリ作成
  tmp_dir=$(mktemp -d)
  trap 'rm -rf "$tmp_dir"' EXIT

  # アーカイブダウンロード
  echo "Downloading wmgr archive..."
  if ! curl -sSLf "$wmgr_download_url" -o "$tmp_dir/wmgr.tar.gz"; then
    echo "Failed to download wmgr archive from: $wmgr_download_url" 1>&2
    echo "Please check if the version ${WMGR_VERSION} exists and supports ${wmgr_os}/${wmgr_arch}" 1>&2
    return 1
  fi

  # アーカイブ展開
  echo "Extracting wmgr archive..."
  if ! tar -xzf "$tmp_dir/wmgr.tar.gz" -C "$tmp_dir"; then
    echo "Failed to extract wmgr archive" 1>&2
    return 1
  fi

  # バイナリ配置
  echo "Installing wmgr to $wmgr_install_path/$wmgr_binary"
  if ! cp "$tmp_dir/$wmgr_binary" "$wmgr_install_path/$wmgr_binary"; then
    echo "Failed to install wmgr binary. Check permissions for $wmgr_install_path" 1>&2
    echo "You may need to run this script with sudo or choose a different install path" 1>&2
    return 1
  fi
  
  chmod 755 -- "$wmgr_install_path/$wmgr_binary"

  echo ""
  echo "✅ wmgr ${WMGR_VERSION} has been successfully installed!"
  echo ""
  echo "The wmgr binary is installed at: $wmgr_install_path/$wmgr_binary"
  echo ""
  echo "To get started, run:"
  echo "  wmgr --help"
  echo ""
  echo "For more information, visit: https://github.com/${WMGR_REPO}"
}

main "$@"