"""Build the signed universal app without embedding local home-folder paths."""
import os
from pathlib import Path
import shlex
import subprocess

root = Path(__file__).resolve().parents[1]
env = os.environ.copy()
if not env.get("APPLE_SIGNING_IDENTITY"):
    raise SystemExit("Set APPLE_SIGNING_IDENTITY to your Developer ID Application identity.")
flags = (
    env["CARGO_ENCODED_RUSTFLAGS"].split("\x1f")
    if env.get("CARGO_ENCODED_RUSTFLAGS")
    else shlex.split(env.get("RUSTFLAGS", ""))
)
flags += [
    "--remap-path-prefix", f"{Path.home()}=/build/home",
    "--remap-path-prefix", f"{root}=/build/dlive-reaper-link",
]
env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
env.setdefault("CARGO_TARGET_DIR", "/tmp/dlive-compact-target")
subprocess.run(
    ["npm", "run", "tauri", "build", "--", "--target", "universal-apple-darwin", "--bundles", "app"],
    cwd=root, env=env, check=True,
)
