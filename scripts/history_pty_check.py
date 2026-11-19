"""Linux agent-driven terminal check. Uses the real binary and standard-library PTY."""
import os
from pathlib import Path
import pty
import select
import signal
import time

root = Path(__file__).resolve().parents[1]
binary = root / "target/release/lupine"
pid, fd = pty.fork()
if pid == 0:
    env = dict(os.environ, TERM="xterm")
    os.execve(binary, [str(binary), "play", str(root / "story_example")], env)

pending = b""
transcript = bytearray()
def expect(marker):
    global pending
    deadline = time.monotonic() + 15
    while marker not in pending:
        assert time.monotonic() < deadline, f"Missing {marker!r}: {bytes(transcript)!r}"
        if not select.select([fd], [], [], 0.1)[0]:
            continue
        chunk = os.read(fd, 65536)
        assert chunk, f"Unexpected EOF before {marker!r}"
        transcript.extend(chunk)
        pending += chunk
        if b"\x1b[6n" in pending:
            pending = pending.replace(b"\x1b[6n", b"")
            os.write(fd, b"\x1b[1;1R")
    pending = pending.split(marker, 1)[1]

try:
    expect(b"> ")
    os.write(fd, b"look\n")
    expect(b"Test Foyer")
    expect(b"> ")
    # Recall at the oldest boundary, move inside the line, delete one o, submit lok.
    os.write(fd, b"\x1b[A\x1b[A\x1b[D\x1b[D\x7f\x1b[C\x1b[C\n")
    expect(b"Did you mean 'look'?")
    expect(b"> ")
    # Edit the recalled typo back into look.
    os.write(fd, b"\x1b[A\x1b[Do\x1b[C\n")
    expect(b"Test Foyer")
    expect(b"> ")
    # Down at the newest boundary restores the empty draft.
    os.write(fd, b"\x1b[A\x1b[Binventory\n")
    expect(b"You are carrying nothing.")
    expect(b"> ")
    os.write(fd, b"\x04")
    expect(b"Goodbye!")
    _, status = os.waitpid(pid, 0)
    assert os.waitstatus_to_exitcode(status) == 0
    pid = 0
    print("Linux PTY: Up/Down, boundaries, cursor movement, backspace, recalled edits and EOF passed.")
finally:
    (root / "target/history-pty-transcript.bin").write_bytes(transcript)
    os.close(fd)
    if pid:
        os.kill(pid, signal.SIGTERM)
        os.waitpid(pid, 0)
