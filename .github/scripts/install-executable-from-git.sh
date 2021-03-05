#!/bin/sh

set -e

help() {
    cat <<'EOF'
Install a binary release of a Rust crate hosted on GitHub

Usage:
    install.sh [options]

Options:
    -h, --help      Display this message
    --git SLUG      Get the crate from "https://github/$SLUG"
    -f, --force     Force overwriting an existing binary
    --crate NAME    Name of the crate to install (default <repository name>)
    --tag TAG       Tag (version) of the crate to install (default <latest release>)
    --target TARGET Install the release compiled for $TARGET (default <`rustc` host>)
    --to LOCATION   Where to install the binary (default ~/.cargo/bin)
    --archive_type  Release archive type (.tar.gz or .zip) (default .zip)
    --exec-name     Executable file to be installed
EOF
}

say() {
    echo "install.sh: $1"
}

say_err() {
    say "$1" >&2
}

err() {
    if [ -n "$td" ]; then
        rm -rf "$td"
    fi

    say_err "ERROR $1"
    exit 1
}

need() {
    if ! command -v "$1" > /dev/null 2>&1; then
        err "need $1 (command not found)"
    fi
}

force=false
while test $# -gt 0; do
    case $1 in
        --crate)
            crate=$2
            shift
            ;;
        --force | -f)
            force=true
            ;;
        --git)
            git=$2
            shift
            ;;
        --help | -h)
            help
            exit 0
            ;;
        --tag)
            tag=$2
            shift
            ;;
        --target)
            target=$2
            shift
            ;;
        --to)
            dest=$2
            shift
            ;;
        --archive-type)
            archive_type=$2
            shift
            ;;
        --exec-name)
            exec_name=$2
            shift
            ;;
        *)
            ;;
    esac
    shift
done

# Dependencies
need basename
need curl
need install
need mkdir
need mktemp

# Optional dependencies
if [ -z "$crate" ] || [ -z "$tag" ] || [ -z "$target" ]; then
    need cut
fi

if [ -z "$tag" ]; then
    need rev
fi

if [ -z "$target" ]; then
    need grep
    need rustc
fi

if [ -z "$archive_type" ]; then
  archive_type=".tar.gz"
fi

if [ $archive_type = ".tar.gz" ]; then
  need tar
elif [ $archive_type = ".zip" ]; then
  need unzip
else
  err "Must specify valid and supported archive type (is $archive_type)"
fi

if [ -z "$git" ]; then
    err 'must specify a git repository using `--git`. Example: `install.sh --git japaric/cross`'
fi

url="https://github.com/$git"
say_err "GitHub repository: $url"

if [ -z "$crate" ]; then
    crate=$(echo "$git" | cut -d'/' -f2)
fi

say_err "Crate: $crate"

url="$url/releases"

if [ -z "$tag" ]; then
    tag=$(curl -s "$url/latest" | cut -d'"' -f2 | rev | cut -d'/' -f1 | rev)
    say_err "Tag: latest ($tag)"
else
    say_err "Tag: $tag"
fi

if [ -z "$target" ]; then
    target=$(rustc -Vv | grep host | cut -d' ' -f2)
fi

say_err "Target: $target"

if [ -z "$dest" ]; then
    dest="$HOME/.cargo/bin"
fi

say_err "Installing to: $dest"

url="$url/download/$tag/$crate.$tag.$target$archive_type"

td=$(mktemp -d || mktemp -d -t tmp)
if [ $archive_type = ".tar.gz" ]; then
  echo "curl -sL $url | tar -C $td -xz"
  curl -sL "$url" | tar -C "$td" -xz
elif [ $archive_type = ".zip" ]; then
  curl -o "$td/temp.zip" -sL "$url"
  unzip -o "$td/temp.zip" -d $td
  rm "$td/temp.zip"
fi

if [ -z "$exec_name" ]; then
  for f in $(ls $td); do
      test -x "$td/mdbook-linkcheck" || continue

      if [ -e "$dest/mdbook-linkcheck" ] && [ $force = false ]; then
          err "$f already exists in $dest"
      else
          mkdir -p "$dest"
          install -m 755 "$td/$f" "$dest"
      fi
  done
else
    mkdir -p "$dest"
    install -m 755 "$td/$exec_name" "$dest"
fi

rm -rf "$td"
