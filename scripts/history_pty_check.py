"""Linux agent-driven terminal check. Uses the real binary and standard-library PTY."""
import os
from pathlib import Path
import pty
import select
import signal
import time
import uuid

root = Path(__file__).resolve().parents[1]
binary = root / "target/release/lupine"
userdata = root / "target" / f"prompt-pty-{uuid.uuid4().hex}"
pid, fd = pty.fork()
if pid == 0:
    env = dict(os.environ, TERM="xterm", XDG_DATA_HOME=str(userdata))
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
    # Questions must end on their own line before rustyline redraws the input.
    os.write(fd, b"save\n")
    expect(b"Save name:\r\n")
    expect(b"> ")
    os.write(fd, b"cancel\n")
    expect(b"Save canceled.")
    expect(b"> ")
    assert not userdata.exists(), "Canceling naming must not save"
    os.write(fd, b"save\n")
    expect(b"Save name:\r\n")
    expect(b"> ")
    os.write(fd, b"camp\n")
    expect(b"Saved:")
    expect(b"> ")
    slots = list(userdata.rglob("*.rork"))
    assert len(slots) == 1
    saved = slots[0].read_bytes()
    os.write(fd, b"take key\n")
    expect(b"You take the brass key.")
    expect(b"> ")
    os.write(fd, b"save\n")
    expect(b"Overwrite current save 'camp'? (yes/no):\r\n")
    expect(b"> ")
    os.write(fd, b"no\n")
    expect(b"Save name:\r\n")
    expect(b"> ")
    os.write(fd, b"cancel\n")
    expect(b"Save canceled.")
    expect(b"> ")
    os.write(fd, b"load camp\n")
    expect(b"Discard unsaved progress and load? (yes/no):\r\n")
    expect(b"> ")
    os.write(fd, b"no\n")
    expect(b"Load canceled.")
    expect(b"> ")
    os.write(fd, b"exit\n")
    expect(b"Unsaved progress. Quit without saving? (yes/no):\r\n")
    expect(b"> ")
    # Exercise redraw/editing of an answer without losing its question above it.
    os.write(fd, b"nox\x7f\n")
    expect(b"> ")
    os.write(fd, b"inventory\n")
    expect(b"You are carrying: brass key")
    expect(b"> ")
    assert slots[0].read_bytes() == saved, "Declines must preserve the saved snapshot"
    os.write(fd, b"save\n")
    expect(b"Overwrite current save 'camp'? (yes/no):\r\n")
    expect(b"> ")
    os.write(fd, b"yes\n")
    expect(b"Saved:")
    expect(b"> ")
    assert slots[0].read_bytes() != saved, "Confirmed save must update the current slot"
    saved = slots[0].read_bytes()
    os.write(fd, b"save\n")
    expect(b"Overwrite current save 'camp'? (yes/no):\r\n")
    expect(b"> ")
    os.write(fd, b"no\n")
    expect(b"Save name:\r\n")
    expect(b"> ")
    os.write(fd, b"forest camp\n")
    expect(b"Saved: forestcamp.")
    expect(b"> ")
    assert len(list(userdata.rglob("*.rork"))) == 2
    os.write(fd, b"\x04")
    expect(b"Goodbye!")
    _, status = os.waitpid(pid, 0)
    assert os.waitstatus_to_exitcode(status) == 0
    pid = 0
    assert slots[0].read_bytes() == saved, "EOF must not save"
    print("Linux PTY: history/editing, visible prompts, current-slot overwrite/save-as, declines and EOF passed.")
finally:
    (root / "target/history-pty-transcript.bin").write_bytes(transcript)
    os.close(fd)
    if pid:
        os.kill(pid, signal.SIGTERM)
        os.waitpid(pid, 0)
