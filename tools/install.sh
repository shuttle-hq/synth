#!/bin/sh -e

# NOTE: Basically copy/pasted from the wasp-lang (https://github.com/wasp-lang/wasp) install script. Thanks wasp guys!
# https://github.com/wasp-lang/wasp/blob/75e17f4dc42451e59fee5c93b9537443f19bc63d/waspc/tools/install.sh
SYNTH_TEMP_DIR=
FORCE=
CI=

RED="\033[31m"
GREEN="\033[32m"
BOLD="\033[1m"
RESET="\033[0m"

while [ $# -gt 0 ]; do
  case "$1" in
  -f | --force)
    FORCE="true"
    shift
    ;;
  -c | --ci)
    CI="true"
    shift
    ;;
  *)
    echo "Invalid argument: $1" >&2
    exit 1
    ;;
  esac
done

main() {
  trap cleanup_temp_dir EXIT
  install_based_on_os
}

install_based_on_os() {
  case "$(uname)" in
  "Linux")
    HOME_LOCAL_BIN="$HOME/.local/bin"

    # Assuming [ ${GLIBC_MAJOR} -e 2 ]...
    GLIBC_MINOR=$(ldd --version 2>/dev/null | head -n1 | grep -o -e "[0-9]\{2\}$")

    if [ ! ${GLIBC_MINOR} ]; then
      die "Could not determine local version of glibc"
    fi

    if [ ${GLIBC_MINOR} -lt 29 ]; then
      if [ ${GLIBC_MINOR} -lt 27 ]; then
        die "Sorry, this installer does not support versions of glibc < 2.27"
      fi
      os="18.04"
    else
      os="latest"
    fi

    install_from_bin_package "synth-ubuntu-${os}-x86_64.tar.gz"
    ;;
  "Darwin")
    HOME_LOCAL_BIN="/usr/local/bin"

    os="latest"

    install_from_bin_package "synth-macos-${os}-x86_64.tar.gz"
    ;;
  *)
    die "Sorry, this installer does not support your operating system: $(uname)."
    ;;
  esac
}

prompt_install_telemetry() {
  info "\nTelemetry is disabled by default (more details here: https://getsynth.github.io/synth/other/telemetry)"
  info "Help us improve the product by allowing Synth to collect de-identified command usage data? (y/N) "
  read INSTALL_TELEMETRY </dev/tty

  if [ "$INSTALL_TELEMETRY" = "y" ]; then
    "$BIN_DST_DIR"/synth telemetry enable
  fi
}

# Download a Synth binary package and install it in $HOME_LOCAL_BIN.
install_from_bin_package() {
  PACKAGE_URL="https://github.com/getsynth/synth/releases/latest/download/$1"
  make_temp_dir
  info "Downloading binary package to temporary dir and unpacking it there...\n"
  dl_to_file "$PACKAGE_URL" "$SYNTH_TEMP_DIR/$1"
  echo ""
  mkdir -p "$SYNTH_TEMP_DIR/synth"
  if ! tar xzf "$SYNTH_TEMP_DIR/$1" -C "$SYNTH_TEMP_DIR/synth"; then
    die "Unpacking binary package failed."
  fi

  BIN_DST_DIR="$HOME_LOCAL_BIN"
  create_dir_if_missing "$BIN_DST_DIR"

  # If our install locations are already occupied (by previous synth installation or something else),
  # Inform user that they have to clean it up (or if FORCE is set, we do it for them).

  if [ -e "$BIN_DST_DIR/synth" ]; then
    if [ "$FORCE" = "true" ]; then
      info "Writing over existing $BIN_DST_DIR/synth."
    else
      OCCUPIED_PATH_ERRORS=$OCCUPIED_PATH_ERRORS"Binary file $BIN_DST_DIR/synth already exists.\n"
    fi
  fi
  if [ ! -z "$OCCUPIED_PATH_ERRORS" ]; then
    die "\nInstallation failed!\n${OCCUPIED_PATH_ERRORS}Remove listed entries manually or run the installer with --force flag to write over them:\n  curl -sSL http://sh.getsynth.com | sh -s -- --force"
  fi

  info "Installing Synth executable to $BIN_DST_DIR/synth."

  if ! mv "$SYNTH_TEMP_DIR/synth/synth" "$BIN_DST_DIR/synth"; then
    die "Installing Synth executable to $BIN_DST_DIR failed."
  fi

  if ! chmod +x "$BIN_DST_DIR/synth"; then
    die "Failed to make $BIN_DST_DIR/synth executable."
  fi

  info "${GREEN}Synth has been successfully installed!${RESET}"

  if [ "$CI" != "true" ]; then
    prompt_install_telemetry
  fi

  info "\n${GREEN}Done!${RESET}"

  if ! on_path "$BIN_DST_DIR"; then
    info "\n\n${RED}WARNING${RESET}: It looks like '$BIN_DST_DIR' is not on your PATH! You will not be able to invoke synth from the terminal by its name."
    info "  You can add it to your PATH by adding following line into your profile file (~/.profile or ~/.zshrc or ~/.bash_profile or some other, depending on which shell you use):\n"
    info "  ${BOLD}"'export PATH=$PATH:'"$BIN_DST_DIR${RESET}"
  fi
}

create_dir_if_missing() {
  if [ ! -d "$1" ]; then
    info "$1 does not exist, creating it..."
    if ! mkdir -p "$1" 2>/dev/null; then
      die "Could not create directory: $1."
    fi
  fi
}

# Creates a temporary directory, which will be cleaned up automatically
# when the script finishes
make_temp_dir() {
  SYNTH_TEMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t synth)"
}

# Cleanup the temporary directory if it's been created.
# Called automatically when the script exits.
cleanup_temp_dir() {
  if [ -n "$SYNTH_TEMP_DIR" ]; then
    rm -rf "$SYNTH_TEMP_DIR"
    SYNTH_TEMP_DIR=""
  fi
}

# Print a message to stderr and exit with error code.
die() {
  printf "${RED}$@${RESET}\n" >&2
  exit 1
}

info() {
  printf "$@\n"
}

# Download a URL to file using 'curl' or 'wget'.
dl_to_file() {
  if has_curl; then
    if ! curl ${QUIET:+-sS} --fail -L -o "$2" "$1"; then
      die "curl download failed: $1"
    fi
  elif has_wget; then
    if ! wget ${QUIET:+-q} "-O$2" "$1"; then
      die "wget download failed: $1"
    fi
  else
    die "Neither wget nor curl is available, please install one to continue."
  fi
}

# Check whether 'wget' command exists.
has_wget() {
  has_cmd wget
}

# Check whether 'curl' command exists.
has_curl() {
  has_cmd curl
}

# Check whether the given command exists.
has_cmd() {
  command -v "$1" >/dev/null 2>&1
}

# Check whether the given (query) path is listed in the PATH environment variable.
on_path() {
  # Below we normalize PATH and query regarding ~ by ensuring ~ is expanded to $HOME, avoiding
  # false negatives in case where ~ is expanded in query but not in PATH and vice versa.

  # NOTE: If $PATH or $1 have '|' somewhere in it, sed commands bellow will fail due to using | as their delimiter.

  # If ~ is after : or if it is the first character in the path, replace it with expanded $HOME.
  # For example, if $PATH is ~/martin/bin:~/martin/~tmp/bin,
  # result will be /home/martin/bin:/home/martin/~tmp/bin .
  local PATH_NORMALIZED=$(printf '%s' "$PATH" | sed -e "s|:~|:$HOME|g" | sed -e "s|^~|$HOME|")

  # Replace ~ with expanded $HOME if it is the first character in the query path.
  local QUERY_NORMALIZED=$(printf '%s' "$1" | sed -e "s|^~|$HOME|")

  echo ":$PATH_NORMALIZED:" | grep -q ":$QUERY_NORMALIZED:"
}

main
