#!/usr/bin/env python3
"""Pack one static rmbin binary into a .deb. No maintainer scripts."""

import hashlib
import io
import sys
import tarfile
import time


def ar_member(name: str, data: bytes) -> bytes:
    header = (
        f"{name + '/':<16}"
        f"{int(time.time()):<12}"
        f"{'0':<6}"
        f"{'0':<6}"
        f"{'100644':<8}"
        f"{len(data):<10}"
        "`\n"
    )
    if len(header) != 60:
        raise SystemExit(f"bad ar header length for {name}: {len(header)}")
    payload = data if len(data) % 2 == 0 else data + b"\n"
    return header.encode("ascii") + payload


def gzip_tar(entries: list[tuple[str, bytes | None, int]]) -> bytes:
    buffer = io.BytesIO()
    with tarfile.open(fileobj=buffer, mode="w:gz", format=tarfile.GNU_FORMAT) as archive:
        for name, content, mode in entries:
            info = tarfile.TarInfo(name)
            info.mtime = int(time.time())
            info.mode = mode
            info.uid = 0
            info.gid = 0
            info.uname = "root"
            info.gname = "root"
            if content is None:
                info.type = tarfile.DIRTYPE
                archive.addfile(info)
            else:
                info.size = len(content)
                archive.addfile(info, io.BytesIO(content))
    return buffer.getvalue()


def main() -> None:
    binary_path, version, architecture, output_path = sys.argv[1:5]
    binary = open(binary_path, "rb").read()
    control = (
        f"Package: rmbin\n"
        f"Version: {version}-1\n"
        f"Architecture: {architecture}\n"
        f"Maintainer: pesty-sh <pesty@pesty.sh>\n"
        f"Section: utils\n"
        f"Priority: optional\n"
        f"Description: Offline file and folder remover\n"
        f" Overwrites file contents with a repeating text pattern, syncs to\n"
        f" disk, then deletes the name. No network access and no telemetry.\n"
        f" Overwriting is best effort on SSDs, copy-on-write filesystems,\n"
        f" snapshots, and backups. This package only installs /usr/bin/rmbin.\n"
    ).encode()
    md5sums = f"{hashlib.md5(binary).hexdigest()}  usr/bin/rmbin\n".encode()
    control_tar = gzip_tar(
        [
            ("control", control, 0o644),
            ("md5sums", md5sums, 0o644),
        ]
    )
    data_tar = gzip_tar(
        [
            ("usr", None, 0o755),
            ("usr/bin", None, 0o755),
            ("usr/bin/rmbin", binary, 0o755),
        ]
    )
    deb = (
        b"!<arch>\n"
        + ar_member("debian-binary", b"2.0\n")
        + ar_member("control.tar.gz", control_tar)
        + ar_member("data.tar.gz", data_tar)
    )
    with open(output_path, "wb") as output:
        output.write(deb)


if __name__ == "__main__":
    main()
