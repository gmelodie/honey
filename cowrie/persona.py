import pickle
import re
from importlib.resources import files

A_NAME, A_SIZE, A_CONTENTS = 0, 4, 7

USER = b"deploy"
LASTCHG = b"19782"
OLD_HASH = rb"\$6\$ErqInBoz\$[^:]+"
NEW_HASH = b"$6$/r7Z7Abv3xrozyr.$13voPTa8QyyodoTy0axh9/.vxpAW.UWRMZX9OfDvRwovqM4WQsH2Tx4rBHqvIkuY0xNLytDp6Dyqs/Ag98V5F0"

OS_RELEASE = b"""PRETTY_NAME="Debian GNU/Linux 12 (bookworm)"
NAME="Debian GNU/Linux"
VERSION_ID="12"
VERSION="12 (bookworm)"
VERSION_CODENAME=bookworm
ID=debian
HOME_URL="https://www.debian.org/"
SUPPORT_URL="https://www.debian.org/support"
BUG_REPORT_URL="https://bugs.debian.org/"
"""


def node(fs, path):
    n = fs
    for part in path.strip("/").split("/"):
        n = next(c for c in n[A_CONTENTS] if c[A_NAME] == part)
    return n


def edit(fs, path, fn):
    n = node(fs, path)
    n[A_CONTENTS] = fn(n[A_CONTENTS])
    n[A_SIZE] = len(n[A_CONTENTS])


def rename_user(b):
    b = b.replace(b"Phil California", USER)
    return re.sub(rb"\bphil\b", USER, b)


def drop_lines(prefix):
    return lambda b: b"".join(
        l for l in b.splitlines(keepends=True) if not l.startswith(prefix)
    )


def write(hostname, out):
    fs = pickle.loads((files("cowrie.data") / "fs.pickle").read_bytes())
    host = hostname.encode()

    edit(fs, "/etc/hostname", lambda _: host + b"\n")
    edit(fs, "/etc/hosts", lambda b: b.replace(b"nas3", host))
    edit(fs, "/usr/lib/os-release", lambda _: OS_RELEASE)
    for path in ("/etc/passwd", "/etc/group", "/etc/shadow"):
        edit(fs, path, rename_user)
    edit(fs, "/etc/group", drop_lines(b"vboxsf:"))
    edit(fs, "/etc/shadow", lambda b: re.sub(rb":15800:", b":" + LASTCHG + b":", b))
    edit(fs, "/etc/shadow", lambda b: re.sub(OLD_HASH, lambda _: NEW_HASH, b))
    node(fs, "/home/phil")[A_NAME] = USER.decode()

    with open(out, "wb") as f:
        pickle.dump(fs, f)
