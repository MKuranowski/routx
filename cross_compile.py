#!/usr/bin/env python
# pyright: strict

import os
import shlex
import shutil
import subprocess
import sys
from argparse import ArgumentParser
from collections.abc import Iterable, Mapping
from pathlib import Path
from typing import NamedTuple, cast


class Target(NamedTuple):
    cmd: str
    env: Mapping[str, str] = {}


TARGET_TO_CMD = {
    "aarch64-apple-darwin": Target(
        cmd="cargo zigbuild --target aarch64-apple-darwin",
        env={"MACOSX_DEPLOYMENT_TARGET": "11.0"},
    ),
    "aarch64-pc-windows-msvc": Target("cargo xwin build --target aarch64-pc-windows-msvc"),
    "aarch64-unknown-linux-gnu": Target("cargo zigbuild --target aarch64-unknown-linux-gnu.2.17"),
    "aarch64-unknown-linux-musl": Target(
        cmd="cargo zigbuild --target aarch64-unknown-linux-musl",
        env={"RUSTFLAGS": "-C target-feature=-crt-static"},
    ),
    "x86_64-apple-darwin": Target(
        cmd="cargo zigbuild --target x86_64-apple-darwin",
        env={"MACOSX_DEPLOYMENT_TARGET": "11.0"},
    ),
    "x86_64-pc-windows-msvc": Target("cargo xwin build --target x86_64-pc-windows-msvc"),
    "x86_64-unknown-linux-gnu": Target("cargo zigbuild --target x86_64-unknown-linux-gnu.2.17"),
    "x86_64-unknown-linux-musl": Target(
        cmd="cargo zigbuild --target x86_64-unknown-linux-musl",
        env={"RUSTFLAGS": "-C target-feature=-crt-static"},
    ),
}

# Parse arguments
arg_parser = ArgumentParser()
arg_parser.add_argument("--release", action="store_true")
arg_parser.add_argument("--output", type=Path)
arg_parser.add_argument("targets", nargs="*", choices=TARGET_TO_CMD, default=TARGET_TO_CMD)
args = arg_parser.parse_args()

# Unpack arguments
targets = cast(Iterable[str], args.targets)
release = cast(bool, args.release)
output = cast(Path | None, args.output)

# Process each target
for target in targets:
    cmd, env_extras = TARGET_TO_CMD[target]

    # Prepare the command to execute
    args = shlex.split(cmd)
    executable = shutil.which(args[0])
    if executable is None:
        raise RuntimeError(f"{args[0]!r} not found in PATH")
    args[0] = executable
    if release:
        args.append("--release")

    env = os.environ.copy()
    env.update(**env_extras)

    # Execute the build program
    print("+", *args, file=sys.stderr)
    subprocess.run(args, env=env, check=True)

    # Copy out the files to output directory
    if output:
        print("+", "mkdir", "-p", output, file=sys.stderr)
        output.mkdir(parents=True, exist_ok=True)

        if "-windows-" in target:
            files = ["routx.dll", "routx.lib"]
        elif "-apple-" in target:
            files = ["libroutx.dylib", "libroutx.a"]
        else:
            files = ["libroutx.so", "libroutx.a"]

        artifacts_dir = Path("target", target, "release" if release else "debug")
        for file in files:
            basename, _, extension = file.partition(".")
            src = artifacts_dir / file
            dst = output / f"{basename}-{target}.{extension}"
            print("+", "cp", src, dst, file=sys.stderr)
            shutil.copy2(src, dst)
