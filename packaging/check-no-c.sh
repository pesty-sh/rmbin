#!/bin/sh
# Fail when the build would compile C, or when a crate name ends in -sys.
# Rust's own standard library still calls the operating system. That is not
# a third-party C library.
set -eu

cd "$(dirname "$0")/.."

target="${1:-}"
if [ -n "$target" ]; then
    names=$(cargo tree --locked -e normal,build,dev --prefix none --target "$target" | awk '{print $1}' | sort -u)
else
    names=$(cargo tree --locked -e normal,build,dev --prefix none | awk '{print $1}' | sort -u)
fi

if printf '%s\n' "$names" | grep -Ex 'cc|cmake|bindgen|openssl-sys|libz-sys|onig_sys'; then
    echo "C build crate is not allowed" >&2
    exit 1
fi

if printf '%s\n' "$names" | grep -E -- '-sys$'; then
    echo "a crate name ends in -sys; review it before allowing it" >&2
    exit 1
fi

if find . -name build.rs -not -path './target/*' -not -path './vendor/*' | grep -q .; then
    echo "build.rs is not allowed in rmbin" >&2
    exit 1
fi

if find src tests -type f \( -name '*.c' -o -name '*.cc' -o -name '*.cpp' -o -name '*.h' -o -name '*.S' \) | grep -q .; then
    echo "C or C++ source is not allowed" >&2
    exit 1
fi

echo "no C build scripts and no -sys crates"
