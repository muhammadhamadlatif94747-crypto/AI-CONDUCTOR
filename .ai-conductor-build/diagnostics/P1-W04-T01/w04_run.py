import os
import subprocess

ROOT = r"e:\AI-CONDUCTOR"
env = dict(os.environ)
env["CARGO_TARGET_DIR"] = os.path.join(ROOT, "PROJECT", "target-w04")

result_path = os.path.join(ROOT, "w04_result.txt")
status_path = os.path.join(ROOT, "w04_status.txt")

if os.path.exists(result_path):
    os.remove(result_path)

with open(status_path, "w") as f:
    f.write("STARTED\n")

try:
    r = subprocess.run(
        ["cargo", "test", "--manifest-path", "PROJECT/Cargo.toml"],
        capture_output=True,
        text=True,
        cwd=ROOT,
        env=env,
        timeout=1200,
    )
    out = "EXIT=%d\n===STDOUT===\n%s\n===STDERR===\n%s" % (r.returncode, r.stdout, r.stderr)
    with open(result_path, "w", encoding="utf-8") as f:
        f.write(out)
    with open(status_path, "a") as f:
        f.write("FINISHED exit=%d\n" % r.returncode)
except subprocess.TimeoutExpired as e:
    with open(result_path, "w", encoding="utf-8") as f:
        f.write("TIMEOUT\npartial stdout:\n%s" % (e.stdout,))
    with open(status_path, "a") as f:
        f.write("TIMEOUT\n")
