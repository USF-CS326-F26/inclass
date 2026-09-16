#!/usr/bin/env python3
"""pointat -- put each in-class example beside the output it printed.

For one week of this repo, pointat builds every program in
weekNN/examples/src/bin/, runs it the way its `//! Run:` line says, compiles
every deliberately broken file in weekNN/examples/broken/ the way
show-errors.sh does, and writes weekNN/examples.html: one self-contained page
(no network, works from file://) with the code on the left and what it printed
on the right.

The mapping between the two sides is the point of the page:

  * Sections.  Every program prints `== title ==` headers from main().  The
    k-th header println! starts code section k; the k-th `== ` line of the
    output starts output section k.  Each section is one row of the page, so
    a block of code and the block of output it produced sit side by side.
  * Lines.  Each println!/print! format string becomes a regex.  An output
    line that matches exactly one print site (or one clearly more specific
    than the rest) is linked to it: click either side to see the other.
    Matches that are not certain are never drawn.

Captured output is stored in weekNN/examples/.pointat/captures.json, so the
page can be re-rendered without a Rust toolchain (--no-run), and a checkout on
another machine produces the same page.

Usage:
    python3 tools/pointat.py week04               build, run, write week04/examples.html
    python3 tools/pointat.py week02 week03 week04
    python3 tools/pointat.py --all                every weekNN/ with examples/Cargo.toml
    python3 tools/pointat.py week04 --no-run      re-render from captures.json
    python3 tools/pointat.py --all --check        exit 1 if anything below is off
    python3 tools/pointat.py week04 --dump        print the derived model as JSON
    python3 tools/pointat.py --all --publish ../USF-CS326-F26.github.io
    python3 tools/pointat.py week04 --timeout 10

--check fails when: a crate does not build; a program exits non-zero or times
out; a program's output is missing one of the `== ` headers its source prints,
or has a `== ` line no header explains; a broken file compiles, or its errors
lack the code in its file name; a capture contains this checkout's absolute
path; or a source file changed since it was captured.

--publish SITE writes SITE/docs/inclass/weekNN-examples.html and refreshes
SITE/docs/inclass/weekNN-slides.html from weekNN/slides.html, swapping the
back-link and the examples links for the site's names.  It refuses to replace
a published deck that differs from the source in any other way.

Standard library only; Python 3.9 or newer.  Needs cargo and rustc on PATH
unless --no-run.
"""

from __future__ import annotations

import argparse
import bisect
import datetime
import hashlib
import html
import json
import os
import platform
import re
import shlex
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path
from string import Template

TOOL = "pointat/1"
HERE = Path(__file__).resolve().parent
REPO = HERE.parent


def log(msg: str) -> None:
    print(msg, file=sys.stderr, flush=True)


# ---------------------------------------------------------------------------
# Lexing: one left-to-right pass, shared by highlighting and structure scans
# ---------------------------------------------------------------------------

KEYWORDS = frozenset("""
    as async await break const continue crate dyn else enum extern fn for if
    impl in let loop match mod move mut pub ref return self Self static struct
    super trait type unsafe use where while
""".split())
LITERALS = frozenset(("true", "false"))
PRIMITIVES = frozenset(
    "u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64 bool char str".split())

NUM_RE = re.compile(
    r"(?:0x[0-9a-fA-F_]+|0o[0-7_]+|0b[01_]+|\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d[\d_]*)?)"
    r"(?:_?(?:[ui](?:8|16|32|64|128|size)|f32|f64))?")
IDENT_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
RAW_STR_RE = re.compile(r'b?r(#*)"')


def _ident_char(ch: str) -> bool:
    return ch.isalnum() or ch == "_"


def _scan_string(src: str, j: int) -> int:
    """`j` is just past an opening quote; return the offset past the closing one."""
    n = len(src)
    while j < n:
        ch = src[j]
        if ch == "\\":
            j += 2
        elif ch == '"':
            return j + 1
        else:
            j += 1
    return n


def _scan_char(src: str, i: int) -> int:
    """`src[i]` is a quote.  Return the end of a char literal there, or 0."""
    n = len(src)
    if i + 1 < n and src[i + 1] == "\\":
        j = src.find("'", i + 3 if src.startswith("\\'", i + 1) else i + 2)
        return j + 1 if 0 < j - i <= 12 else 0
    if i + 2 < n and src[i + 2] == "'" and src[i + 1] not in "\n'":
        return i + 3
    return 0


def _scan_attr(src: str, i: int) -> int:
    """`src[i]` is the `[` of an attribute; return the offset past its `]`."""
    depth, j, n = 0, i, len(src)
    while j < n:
        ch = src[j]
        if ch == '"':
            j = _scan_string(src, j + 1)
            continue
        if ch == "[":
            depth += 1
        elif ch == "]":
            depth -= 1
            if depth == 0:
                return j + 1
        elif ch == "\n":
            return j
        j += 1
    return n


def lex(src: str) -> list:
    """Tokens worth colouring, as (start, end, class), in order.

    Classes: c comment, d doc comment, s string, ch char, lt lifetime,
    n number or true/false, a attribute, m macro, k keyword, t type,
    cn CONSTANT.  Identifiers and punctuation are not tokens.
    """
    toks = []
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if c == "/" and src.startswith("//", i):
            j = src.find("\n", i)
            j = n if j < 0 else j
            doc = src.startswith("//!", i) or (src.startswith("///", i) and not src.startswith("////", i))
            toks.append((i, j, "d" if doc else "c"))
            i = j
            continue
        if c == "/" and src.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if src.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif src.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            toks.append((i, j, "c"))
            i = j
            continue
        if c == '"':
            j = _scan_string(src, i + 1)
            toks.append((i, j, "s"))
            i = j
            continue
        prev_ident = i > 0 and _ident_char(src[i - 1])
        if c in "br" and not prev_ident:
            m = RAW_STR_RE.match(src, i)
            if m:
                close = '"' + m.group(1)
                j = src.find(close, m.end())
                j = n if j < 0 else j + len(close)
                toks.append((i, j, "s"))
                i = j
                continue
            if src.startswith('b"', i):
                j = _scan_string(src, i + 2)
                toks.append((i, j, "s"))
                i = j
                continue
            if src.startswith("b'", i):
                j = _scan_char(src, i + 1)
                if j:
                    toks.append((i, j, "ch"))
                    i = j
                    continue
        if c == "'":
            j = _scan_char(src, i)
            if j:
                toks.append((i, j, "ch"))
                i = j
                continue
            m = IDENT_RE.match(src, i + 1)
            if m:
                toks.append((i, m.end(), "lt"))
                i = m.end()
                continue
            i += 1
            continue
        if c == "#" and (src.startswith("#[", i) or src.startswith("#![", i)):
            j = _scan_attr(src, src.index("[", i))
            toks.append((i, j, "a"))
            i = j
            continue
        if c.isdigit() and not prev_ident:
            m = NUM_RE.match(src, i)
            toks.append((i, m.end(), "n"))
            i = m.end()
            continue
        if c.isalpha() or c == "_":
            m = IDENT_RE.match(src, i)
            if not m:  # a non-ASCII letter
                i += 1
                continue
            w, j = m.group(), m.end()
            if j < n and src[j] == "!" and not src.startswith("!=", j):
                toks.append((i, j + 1, "m"))
                i = j + 1
                continue
            if w in KEYWORDS:
                cls = "k"
            elif w in LITERALS:
                cls = "n"
            elif w in PRIMITIVES:
                cls = "t"
            elif w[0].isupper():
                cls = "cn" if len(w) > 1 and w.upper() == w else "t"
            else:
                cls = None
            if cls:
                toks.append((i, j, cls))
            i = j
            continue
        i += 1
    return toks


def mask_of(src: str, toks: list) -> str:
    """The source with comment and literal text blanked (newlines kept), so
    braces, parens and macro names can be found by plain scanning."""
    buf = list(src)
    for s, e, cls in toks:
        if cls in ("c", "d", "s", "ch"):
            for k in range(s, e):
                if buf[k] != "\n":
                    buf[k] = " "
    return "".join(buf)


# ---------------------------------------------------------------------------
# Blocks: every `{ ... }`, classified by the statement that opens it
# ---------------------------------------------------------------------------

ATTR_RE = re.compile(r"#!?\[[^\]]*\]")
FN_HEAD_RE = re.compile(r'(?:pub(?:\([^)]*\))?\s+)?(?:(?:const|async|unsafe|extern(?:\s+"[^"]*")?)\s+)*fn\b')
IMPL_HEAD_RE = re.compile(r"(?:unsafe\s+)?impl\b")
ITEM_HEAD_RE = re.compile(r"(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|trait|mod|union)\b")
IMPL_PARTS_RE = re.compile(r"(?:unsafe\s+)?impl(?:\s*<[^{]*?>)?\s+(.+?)(?:\s+for\s+(.+?))?(?:\s+where\b.*)?$")


@dataclass
class Block:
    open: int
    close: int
    kind: str   # fn impl item loop closure arm cond block
    head: str


PATTERN_TAIL_RE = re.compile(r"(?:=>|=(?!=)|in\b|\|)")


def statement_head(mask: str, i: int) -> str:
    """Text of the statement that reaches offset `i` (usually a `{`): back to
    the previous `;`, `{`, `}`, `(` or `[` that is not nested inside a group in
    between.  A `}` closing a struct literal inside `[...]` is part of the head,
    and so is one closing a struct pattern (`for Foo { a } in`, `if let Foo { a }
    =`, `Foo { a } =>`), recognised by what follows it."""
    depth, j = 0, i - 1
    while j >= 0:
        ch = mask[j]
        if ch in ")]":
            depth += 1
        elif ch == "}":
            if depth == 0 and not PATTERN_TAIL_RE.match(mask[j + 1:i].lstrip()):
                break
            depth += 1
        elif ch in "([{":
            if depth == 0:
                break
            depth -= 1
        elif ch == ";" and depth == 0:
            break
        j -= 1
    return mask[j + 1:i]


def classify_head(head: str) -> str:
    if head.endswith("|") or re.search(r"\|\s*->[^|]*$", head):
        return "closure"
    if head.endswith("=>"):
        return "arm"
    if FN_HEAD_RE.match(head):
        return "fn"
    if IMPL_HEAD_RE.match(head):
        return "impl"
    if ITEM_HEAD_RE.match(head):
        return "item"
    if re.search(r"\b(?:for|while|loop)\b", head):
        return "loop"
    if re.search(r"\b(?:if|else|match)\b", head):
        return "cond"
    return "block"


def find_blocks(mask: str) -> list:
    blocks, stack = [], []
    for i, ch in enumerate(mask):
        if ch == "{":
            head = " ".join(ATTR_RE.sub(" ", statement_head(mask, i)).split())
            blocks.append(Block(i, len(mask), classify_head(head), head))
            stack.append(len(blocks) - 1)
        elif ch == "}" and stack:
            blocks[stack.pop()].close = i
    return blocks


def matching_paren(mask: str, i: int) -> int:
    depth = 0
    for j in range(i, len(mask)):
        if mask[j] == "(":
            depth += 1
        elif mask[j] == ")":
            depth -= 1
            if depth == 0:
                return j
    return len(mask)


class Source:
    """A Rust file: text, tokens, the mask, line offsets and blocks."""

    def __init__(self, path: Path, text: str | None = None):
        self.path = path
        self.name = path.name
        self.text = path.read_text(encoding="utf-8") if text is None else text
        self.toks = lex(self.text)
        self.mask = mask_of(self.text, self.toks)
        self.starts = [0] + [m.end() for m in re.finditer("\n", self.text)]
        self.lines = self.text.split("\n")
        if len(self.lines) > 1 and self.lines[-1] == "":
            self.lines.pop()
        self.tok_at = {t[0]: t for t in self.toks}
        self.blocks = find_blocks(self.mask)

    def line_of(self, off: int) -> int:
        return bisect.bisect_right(self.starts, off)

    def enclosing(self, off: int) -> list:
        """Blocks containing `off`, outermost first."""
        return [b for b in self.blocks if b.open < off < b.close]

    def main_block(self) -> Block | None:
        for b in self.blocks:
            if b.kind == "fn" and re.search(r"\bfn\s+main\s*\(", b.head):
                return b
        return None


# ---------------------------------------------------------------------------
# Format strings: literal -> text -> per-line regexes
# ---------------------------------------------------------------------------

PH_RE = re.compile(r"\{\{|\}\}|\{([^{}]*)\}")
PH_CLASS = {
    "p": r"0x[0-9a-fA-F]+",
    "#x": r"0x[0-9a-f]+",
    "#X": r"0x[0-9A-F]+",
    "#o": r"0o[0-7]+",
    "#b": r"0b[01]+",
}


def unescape(lit: str) -> str:
    """The text of a Rust string literal (with its quotes)."""
    m = RAW_STR_RE.match(lit)
    if m:
        return lit[m.end():len(lit) - 1 - len(m.group(1))]
    body = lit[lit.index('"') + 1:-1]
    out, i, n = [], 0, len(body)
    simple = {"n": "\n", "t": "\t", "r": "\r", "0": "\0", "\\": "\\", '"': '"', "'": "'"}
    while i < n:
        ch = body[i]
        if ch != "\\" or i + 1 >= n:
            out.append(ch)
            i += 1
            continue
        nx = body[i + 1]
        if nx in simple:
            out.append(simple[nx])
            i += 2
        elif nx == "x":
            out.append(chr(int(body[i + 2:i + 4], 16)))
            i += 4
        elif nx == "u":
            k = body.index("}", i)
            out.append(chr(int(body[i + 3:k], 16)))
            i = k + 1
        elif nx == "\n":
            i += 2
            while i < n and body[i] in " \t\r\n":
                i += 1
        else:
            out.append(nx)
            i += 2
    return "".join(out)


@dataclass
class Tmpl:
    text: str
    body: str
    regex: object          # full line, anchored, matched against rstripped text
    lit_chars: int         # non-space literal characters
    nph: int               # placeholders
    pretty: bool           # has a {:#?} placeholder (value spans lines)
    pretty_start: object   # regex for the first line of a pretty value, or None


def build_tmpl(line: str) -> Tmpl:
    t = line.rstrip()
    parts, lit, nph, pre, last = [], 0, 0, None, 0
    for m in PH_RE.finditer(t):
        seg = t[last:m.start()]
        parts.append(re.escape(seg))
        lit += len(seg.replace(" ", ""))
        tok = m.group(0)
        if tok == "{{":
            parts.append(r"\{")
            lit += 1
        elif tok == "}}":
            parts.append(r"\}")
            lit += 1
        else:
            nph += 1
            inner = m.group(1)
            spec = inner.split(":", 1)[1] if ":" in inner else ""
            if pre is None and "#" in spec and spec.endswith("?"):
                pre = list(parts)
            parts.append(PH_CLASS.get(spec, "(.*?)"))
        last = m.end()
    seg = t[last:]
    parts.append(re.escape(seg))
    lit += len(seg.replace(" ", ""))
    body = "".join(parts)
    start = re.compile("^" + "".join(pre) + r"(.*[\[{(])$") if pre is not None else None
    return Tmpl(line, body, re.compile("^" + body + "$"), lit, nph, pre is not None, start)


# ---------------------------------------------------------------------------
# Print sites
# ---------------------------------------------------------------------------

PRINT_RE = re.compile(r"\b(e?print(?:ln)?)!\s*\(")


@dataclass
class Site:
    id: int
    macro: str
    line: int
    end_line: int
    fmt: str | None
    bare: bool
    in_main: bool
    once: bool          # in main and not inside a loop or closure: prints at most once
    cond: bool          # inside an if/match
    header: bool
    lead_blank: int = 0
    lines: list = field(default_factory=list)   # full-line templates
    frag: Tmpl | None = None                    # print!: the unterminated tail
    suffix: object = None                       # println!: its first line as a line tail
    section: int | None = None
    owner: str | None = None                    # e.g. "Tracked::drop()" for sites outside main
    owner_note: str | None = None
    owner_key: int | None = None
    drop: bool = False
    anywhere: bool = False      # in a closure or nested fn in main: may print from any section
    cond_opens: frozenset = frozenset()   # the if/match blocks it sits in

    @property
    def lit_total(self) -> int:
        return sum(t.lit_chars for t in self.lines)

    @property
    def wildcard_only(self) -> bool:
        return bool(self.lines) and self.lit_total == 0

    def pattern_key(self) -> tuple:
        return tuple(t.body for t in self.lines)


def strip_generics(s: str) -> str:
    return re.sub(r"<.*>", "", s).strip()


def owner_of(chain: list) -> tuple:
    """(label, note, key, is_drop) for a print site outside main."""
    fn_b = next((b for b in reversed(chain) if b.kind == "fn"), None)
    if fn_b is None:
        return None, None, None, False
    fm = re.search(r"\bfn\s+(\w+)", fn_b.head)
    fname = fm.group(1) if fm else "?"
    outer = next((b for b in reversed(chain) if b.open < fn_b.open and b.kind in ("impl", "item")), None)
    if outer is not None and outer.kind == "impl":
        m = IMPL_PARTS_RE.match(outer.head)
        if m:
            first, second = m.group(1), m.group(2)
            typ = strip_generics(second or first)
            trait = strip_generics(first) if second else None
            note = f"impl {trait} for {typ}" if trait and trait != "Drop" else None
            return f"{typ}::{fname}()", note, fn_b.open, trait == "Drop"
    if outer is not None and outer.kind == "item":
        m = re.search(r"\btrait\s+(\w+)", outer.head)
        if m:
            return f"{m.group(1)}::{fname}()", "a default method", fn_b.open, False
    return f"{fname}()", None, fn_b.open, False


def find_sites(src: Source, main: Block | None) -> list:
    sites = []
    for m in PRINT_RE.finditer(src.mask):
        start = m.start()
        open_p = m.end() - 1
        close_p = matching_paren(src.mask, open_p)
        j = open_p + 1
        while j < close_p:
            t = src.tok_at.get(j)
            if t and t[2] in ("c", "d"):
                j = t[1]
            elif src.text[j].isspace():
                j += 1
            else:
                break
        t = src.tok_at.get(j)
        fmt = None
        if j < close_p and t and t[2] == "s" and src.text[t[0]] != "b":
            fmt = unescape(src.text[t[0]:t[1]])
        chain = src.enclosing(start)
        in_main = main is not None and main.open < start < main.close
        inner = [b for b in chain if b.open > main.open] if in_main else []
        kinds = [b.kind for b in inner]
        # A body with no braces: `|x| println!(..)` or `A => println!(..)`.
        head = statement_head(src.mask, start).rstrip()
        if head.endswith("|") or re.search(r"\|\s*->[^|]*$", head):
            kinds.append("closure")
        elif head.endswith("=>"):
            kinds.append("arm")
        macro = m.group(1)
        site = Site(
            id=len(sites),
            macro=macro,
            line=src.line_of(start),
            end_line=src.line_of(close_p),
            fmt=fmt,
            bare=j >= close_p,
            in_main=in_main,
            once=in_main and all(k in ("cond", "arm", "block") for k in kinds),
            cond=any(k in ("cond", "arm") for k in kinds),
            header=(in_main and not kinds and macro == "println" and fmt is not None
                    and re.match(r"\n*== ", fmt) is not None),
            anywhere=in_main and any(k in ("closure", "fn") for k in kinds),
            cond_opens=frozenset(b.open for b in inner if b.kind in ("cond", "arm"))
                       | (frozenset([-start]) if kinds and kinds[-1] == "arm" and head.endswith("=>") else frozenset()),
        )
        if fmt is not None:
            parts = fmt.split("\n")
            if macro.endswith("ln"):
                full, frag = parts, None
            else:
                full, frag = parts[:-1], (parts[-1] or None)
            while full and full[0] == "":
                full.pop(0)
                site.lead_blank += 1
            while full and full[-1] == "":
                full.pop()
            site.lines = [build_tmpl(p) for p in full]
            if frag is not None:
                ft = build_tmpl(frag)
                ft.regex = re.compile("^" + ft.body)
                site.frag = ft
            if macro.endswith("ln") and site.lines:
                site.suffix = re.compile("^.*?" + site.lines[0].body + "$")
        if not in_main:
            site.owner, site.owner_note, site.owner_key, site.drop = owner_of(chain)
        sites.append(site)
    return sites


# ---------------------------------------------------------------------------
# Programs and broken files
# ---------------------------------------------------------------------------

RUN_RE = re.compile(r"((?:[A-Za-z_]\w*=\S*\s+)*)cargo run\b(.*)$")
SHELL_OPS = ("|", "||", "&&", ";", "&")
RAW_WRITE_RE = re.compile(r"\be?print!\s*\(|\bstdout\s*\(|\bstderr\s*\(|\bdbg!\s*\(|\bwrite_all\s*\(")


@dataclass
class Program:
    stem: str
    path: Path
    src: Source
    num: str
    title: str
    doc_end: int
    runs: list          # [(cmd, args, env)]
    main_open: int
    main_close: int
    sites: list
    headers: list
    ranges: dict        # section -> (first line, last line); may be empty (lo > hi)
    has_raw: bool


def parse_runs(doc_lines: list, stem: str) -> list:
    """(command as written, program arguments, environment) for each
    `cargo run --bin <stem>` line from the `Run:` line on.  Cargo's own flags
    are ignored (the debug build is run), arguments are what follows `--`,
    `NAME=value` prefixes become environment, and a pipe or redirection ends
    the arguments."""
    runs, seen = [], False
    for line in doc_lines:
        body = line[3:]
        if "Run:" in body:
            seen = True
        if not seen:
            continue
        m = RUN_RE.search(body)
        if not m:
            continue
        try:
            toks = shlex.split(m.group(2))
        except ValueError:
            continue
        if "--bin" not in toks or toks.index("--bin") + 1 >= len(toks) or toks[toks.index("--bin") + 1] != stem:
            continue
        args = toks[toks.index("--") + 1:] if "--" in toks else []
        for k, tok in enumerate(args):
            if tok in SHELL_OPS or tok[:1] in "<>" or tok.startswith(("2>", "1>")):
                args = args[:k]
                break
        env = dict(e.split("=", 1) for e in m.group(1).split())
        runs.append(((m.group(1) + "cargo run" + m.group(2)).strip(), args, env))
    return runs or [("cargo run --bin " + stem, [], {})]


def load_program(path: Path, text: str | None = None) -> Program:
    src = Source(path, text)
    stem = path.stem
    doc_end = 0
    for i, line in enumerate(src.lines, 1):
        if not line.startswith("//!"):
            break
        doc_end = i
    first = src.lines[0][3:].strip() if doc_end else stem
    m = re.match(r"(\d+)\s*[—–-]+\s*(.*)", first)
    num, title = (m.group(1), m.group(2)) if m else (stem.split("_")[0], first)
    main = src.main_block()
    if main is None:
        raise SystemExit(f"{path}: no fn main")
    main_open, main_close = src.line_of(main.open), src.line_of(main.close)
    sites = find_sites(src, main)
    headers = [s for s in sites if s.header]
    starts = []
    for h in headers:
        lo = h.line
        while lo - 1 > main_open and src.lines[lo - 2].strip().startswith("//"):
            lo -= 1
        starts.append(lo)
    ranges = {}
    if headers:
        ranges[0] = (main_open + 1, starts[0] - 1)
        for k, lo in enumerate(starts, 1):
            ranges[k] = (lo, starts[k] - 1 if k < len(starts) else main_close - 1)
    else:
        ranges[0] = (main_open + 1, main_close - 1)
    for s in sites:
        if s.in_main:
            s.section = next((k for k, (lo, hi) in ranges.items() if lo <= s.line <= hi), 0)
    return Program(stem, path, src, num, title, doc_end, parse_runs(src.lines[:doc_end], stem),
                   main_open, main_close, sites, headers, ranges,
                   RAW_WRITE_RE.search(src.mask) is not None)


FIX_RE = re.compile(r"FIX(?:\s+\d+)?:\s*")


@dataclass
class Broken:
    stem: str
    short: str
    code: str | None
    path: Path
    src: Source
    head_end: int
    error: str
    why: list           # paragraphs, each a list of lines
    fixes: list
    note: str


def load_broken(path: Path, text: str | None = None) -> Broken:
    src = Source(path, text)
    stem = path.stem
    m = re.match(r"e(\d{4})", stem)
    code = f"E{m.group(1)}" if m else None
    head = []
    for line in src.lines:
        if not line.startswith("//"):
            break
        head.append(line)
    paras = [[]]
    for line in head:
        body = line[2:]
        if not body.strip():
            if paras[-1]:
                paras.append([])
            continue
        paras[-1].append(body[1:] if body.startswith(" ") else body)
    paras = [p for p in paras if p]
    error = " ".join(x.strip() for x in paras[0]) if paras else ""
    why, fixes, note = [], [], []
    for p in paras[1:]:
        if not any(FIX_RE.match(x) for x in p):
            why.append(p)
            continue
        state, lead = "why", []
        for x in p:
            fm = FIX_RE.match(x)
            if fm:
                fixes.append(x[fm.end():].strip())
                state = "fix"
            elif state == "why":
                lead.append(x)
            elif state == "fix" and re.match(r" {4,}", x):
                fixes[-1] += " " + x.strip()
            else:
                note.append(x.strip())
                state = "note"
        if lead:
            why.append(lead)
    return Broken(stem, stem[:5] if code else stem, code, path, src, len(head), error, why,
                  fixes, " ".join(note))


# ---------------------------------------------------------------------------
# Running: build, run, compile the broken files, keep what they said
# ---------------------------------------------------------------------------

def run_env() -> dict:
    return dict(os.environ, NO_COLOR="1", TERM="dumb", RUST_BACKTRACE="0", CARGO_TERM_COLOR="never")


def tool_version(argv: list) -> str:
    try:
        return subprocess.run(argv, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                              check=False).stdout.decode().strip()
    except OSError:
        return "?"


def os_desc() -> str:
    if platform.system() == "Darwin":
        return f"macOS {platform.mac_ver()[0]} {platform.machine()}"
    return f"{platform.system()} {platform.release()} {platform.machine()}"


def flatten_diag(msg: dict, file_name: str) -> dict:
    """The parts of a rustc JSON diagnostic the page uses.  Spans carrying a
    suggested replacement are where the compiler proposes to edit, not what
    it complains about, so they are dropped, as are zero-width spans."""
    spans = []

    def take(sps: list, child: bool) -> None:
        for sp in sps or []:
            if sp.get("suggested_replacement") is not None:
                continue
            if os.path.basename(sp.get("file_name", "")) != file_name:
                continue
            ls, le, cs, ce = sp["line_start"], sp["line_end"], sp["column_start"], sp["column_end"]
            if ls == le and ce <= cs:
                continue
            spans.append({"ls": ls, "le": le, "cs": cs, "ce": ce,
                          "primary": bool(sp.get("is_primary")) and not child,
                          "label": sp.get("label") or ""})

    take(msg.get("spans"), False)
    for ch in msg.get("children") or []:
        take(ch.get("spans"), True)
    return {"level": msg.get("level"), "code": (msg.get("code") or {}).get("code"),
            "message": msg.get("message", ""), "rendered": msg.get("rendered") or "",
            "spans": spans}


def cargo_build(crate: Path, env: dict) -> tuple:
    p = subprocess.run(["cargo", "build", "--bins", "--message-format=json"], cwd=crate,
                       stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env, check=False)
    diags, exes, success = {}, {}, p.returncode == 0
    for line in p.stdout.decode("utf-8", "replace").splitlines():
        if not line.startswith("{"):
            continue
        try:
            obj = json.loads(line)
        except ValueError:
            continue
        reason = obj.get("reason")
        if reason == "compiler-message":
            msg, target = obj["message"], obj["target"]
            if msg.get("level") not in ("warning", "error"):
                continue
            if not msg.get("spans") and re.match(r"(?:\d+ warnings? emitted|aborting due to)", msg.get("message", "")):
                continue
            name = os.path.basename(target.get("src_path", target["name"] + ".rs"))
            diags.setdefault(target["name"], []).append(flatten_diag(msg, name))
        elif reason == "compiler-artifact" and obj.get("executable"):
            exes[obj["target"]["name"]] = obj["executable"]
        elif reason == "build-finished":
            success = bool(obj.get("success"))
    return success, diags, exes, p.stderr.decode("utf-8", "replace")


def run_program(exe: str, crate: Path, args: list, timeout: float, env: dict) -> dict:
    argv0 = os.path.relpath(exe, crate)   # what `cargo run` shows as argv[0]
    t0 = time.monotonic()
    try:
        p = subprocess.run([argv0, *args], executable=exe, cwd=crate, env=env, timeout=timeout,
                           stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                           check=False)
        raw, code, timed_out = p.stdout, p.returncode, False
    except subprocess.TimeoutExpired as e:
        raw, code, timed_out = e.output or b"", None, True
    out = raw.decode("utf-8", "replace")
    return {"output": out, "exit": code, "timed_out": timed_out,
            "replacement_chars": out.count("\ufffd"), "_seconds": time.monotonic() - t0}


def compile_broken(b: Broken, env: dict) -> dict:
    cmd = ["rustc", "--edition", "2021", "--emit=metadata", "-o", os.devnull, b.path.name]
    try:
        p = subprocess.run(cmd + ["--error-format=json"], cwd=b.path.parent, env=env, timeout=60,
                           stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                           check=False)
    except subprocess.TimeoutExpired:
        return {"cmd": " ".join(cmd), "rendered": "", "diagnostics": [], "exit": None, "timed_out": True}
    rendered, diags = [], []
    for line in p.stderr.decode("utf-8", "replace").splitlines(keepends=True):
        if line.startswith("{"):
            try:
                obj = json.loads(line)
            except ValueError:
                rendered.append(line)
                continue
            if obj.get("$message_type", "diagnostic") != "diagnostic":
                continue
            rendered.append(obj.get("rendered") or "")
            diags.append(flatten_diag(obj, b.path.name))
        else:
            rendered.append(line)
    return {"cmd": " ".join(cmd).replace(os.devnull, "/dev/null"), "rendered": "".join(rendered),
            "diagnostics": diags, "exit": p.returncode, "timed_out": False}


def sha1(text: str) -> str:
    return hashlib.sha1(text.encode("utf-8")).hexdigest()


def capture_week(week: str, programs: list, brokens: list, timeout: float) -> dict:
    crate = REPO / week / "examples"
    env = run_env()
    t0 = time.monotonic()
    success, diags, exes, stderr = cargo_build(crate, env)
    nwarn = sum(1 for ds in diags.values() for d in ds if d["level"] == "warning")
    log(f"{week}: cargo build {'ok' if success else 'FAILED'} in {time.monotonic() - t0:.1f}s"
        f" ({nwarn} warning{'s' if nwarn != 1 else ''})")
    cap = {
        "tool": TOOL, "week": week,
        "captured_at": datetime.datetime.now().astimezone().isoformat(timespec="seconds"),
        "rustc": tool_version(["rustc", "--version"]), "cargo": tool_version(["cargo", "--version"]),
        "os": os_desc(),
        "build": {"success": success, "stderr": "" if success else stderr},
        "programs": {}, "broken": {},
    }
    for p in programs:
        entry = {"source_sha1": sha1(p.src.text), "diagnostics": diags.get(p.stem, []), "runs": []}
        exe = exes.get(p.stem)
        for cmd, args, extra_env in p.runs:
            if exe is None:
                r = {"output": "", "exit": None, "timed_out": False, "replacement_chars": 0, "_seconds": 0}
            else:
                r = run_program(exe, crate, args, timeout, dict(env, **extra_env))
            entry["runs"].append({"cmd": cmd, "args": args, "output": r["output"], "exit": r["exit"],
                                  "timed_out": r["timed_out"], "replacement_chars": r["replacement_chars"]})
        cap["programs"][p.stem] = entry
    for b in brokens:
        c = compile_broken(b, env)
        c["source_sha1"] = sha1(b.src.text)
        cap["broken"][b.stem] = c
    return cap


def captures_path(week: str) -> Path:
    return REPO / week / "examples" / ".pointat" / "captures.json"


def save_captures(week: str, cap: dict) -> dict:
    """Write captures.json unless only the timestamp changed; return what is on disk."""
    path = captures_path(week)
    if path.exists():
        try:
            old = json.loads(path.read_text(encoding="utf-8"))
        except ValueError:
            old = None
        if old is not None and {**old, "captured_at": ""} == {**cap, "captured_at": ""}:
            return old
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(cap, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    return cap


# ---------------------------------------------------------------------------
# Mapping output lines to sections and print sites
# ---------------------------------------------------------------------------

PANIC_RE = re.compile(r"^thread '.*' panicked at (.+?):(\d+):(\d+):?$")


@dataclass
class OutLine:
    text: str
    sec: int
    src: int | None = None
    end: int | None = None
    conf: str = "none"      # confident | inferred | none
    kind: str = ""
    also: list = field(default_factory=list)
    site: int | None = None
    owner: str | None = None
    owner_note: str | None = None
    drop: bool = False


@dataclass
class Mapped:
    lines: list
    starts: dict            # section -> index of its `== ` line
    silent: list            # sections whose header never printed
    unexpected: list        # indices of `== ` lines no header explains


def paren_balance(text: str) -> int:
    t = re.sub(r'"(?:[^"\\]|\\.)*"', '""', text)
    t = re.sub(r"'(?:[^'\\]|\\.)'", "''", t)
    return sum(t.count(c) for c in "([{") - sum(t.count(c) for c in ")]}")


def _assign(ol: OutLine, site: Site, conf: str, kind: str, also=()) -> None:
    ol.src, ol.end, ol.conf, ol.kind, ol.site = site.line, site.end_line, conf, kind, site.id
    ol.also = list(also)
    ol.owner, ol.owner_note, ol.drop = site.owner, site.owner_note, site.drop


def _match_full(site: Site, lines: list, i: int, is_header=None) -> tuple | None:
    """(lines covered, pretty-continuation lines) if `site` printed lines[i...].

    A `{:#?}` value is followed until its brackets balance, but never past a
    line that starts a section: output can arrive truncated or split, and the
    closing bracket may never come, in which case the rest of the stream is
    not this site's."""
    n, k = len(lines), len(site.lines)
    if i + k > n:
        return None
    for j, tm in enumerate(site.lines):
        text = lines[i + j].text.rstrip()
        if tm.regex.match(text):
            continue
        if j == k - 1 and tm.pretty_start is not None and tm.pretty_start.match(text):
            break
        return None
    extra, last = 0, lines[i + k - 1].text
    if site.lines[-1].pretty and paren_balance(last) > 0:
        bal, j = paren_balance(last), i + k
        while j < n and bal > 0:
            if is_header is not None and is_header(lines[j].text):
                return k, 0          # the value never closed; claim nothing after it
            bal += paren_balance(lines[j].text)
            extra += 1
            j += 1
        if bal > 0:
            return k, 0              # ran off the end of a truncated stream
    return k, extra


def _in_section(s: Site, sec: int) -> bool:
    """Could `s` have printed a line of output section `sec`?"""
    return not s.in_main or s.anywhere or s.section == sec


def _terminators(prog: Program, s: Site, tail: str) -> list:
    """Print sites after `s`, in source order, that finished the line `s`
    started.  A fragment with literal text must be found in what is left of
    the line.  A site in an if/match branch that `s` is not in is skipped when
    it cannot be checked, because that branch may not have run.  Empty unless
    the end of the line is accounted for."""
    if s.in_main:
        scope = [x for x in prog.sites if x.in_main and not x.anywhere and x.section == s.section]
    else:
        scope = [x for x in prog.sites if not x.in_main and x.owner_key == s.owner_key]
    also, rest = [], tail
    for x in sorted((x for x in scope if x.line > s.line), key=lambda x: x.line):
        branch = bool(x.cond_opens - s.cond_opens)
        if not x.macro.endswith("ln"):
            fr = x.frag
            if fr is None:
                if branch:
                    continue
                return []
            if fr.lit_chars > 0:
                m = re.search(fr.body, rest)
                if m:
                    also.append(x.line)
                    rest = rest[m.end():]
                    continue
                if branch:
                    continue
                return []
            if not branch:
                also.append(x.line)
            continue
        if x.bare or x.lead_blank:
            return [] if branch else also + [x.line]
        if x.suffix is not None and x.lit_total > 0 and x.suffix.match(rest.rstrip()):
            return also + [x.line]
        if branch:
            continue
        return []
    return []


def _header_start(prog: Program, text: str, nxt: int) -> int | None:
    """The section a `== ` line starts, or None if it is data."""
    for k in range(nxt, len(prog.headers) + 1):
        tm = prog.headers[k - 1].lines[0]
        if not tm.regex.match(text) or (k > nxt and tm.lit_chars < 4):
            continue
        for s in prog.sites:   # a print in the current section that explains it better
            if (not s.header and len(s.lines) == 1 and _in_section(s, nxt - 1)
                    and s.lit_total > tm.lit_chars and s.lines[0].regex.match(text)):
                return None
        return k
    return None


def _downgrade(lines: list, idx: int, kind: str) -> None:
    """Withdraw the link at `idx` and the continuation lines that came with it."""
    site = lines[idx].site
    lines[idx].conf, lines[idx].kind = "inferred", kind
    j = idx + 1
    while j < len(lines) and lines[j].site == site and lines[j].kind in ("cont", "pretty"):
        lines[j].conf = "inferred"
        j += 1


def _check_wildcards(prog: Program, lines: list) -> None:
    """A `{}` whose value spans lines leaves an output line nothing explains,
    and pushes every later wildcard match in its section down by one.  So in a
    section with an unexplained line, no wildcard-only match is trusted."""
    bad = {ol.sec for ol in lines if ol.text.strip() and ol.conf == "none"}
    for idx, ol in enumerate(lines):
        if (ol.sec in bad and ol.conf == "confident" and ol.site is not None
                and prog.sites[ol.site].wildcard_only):
            ol.conf, ol.kind = "inferred", "unexplained-section"


def _check_order(prog: Program, lines: list) -> None:
    """In main, a statement outside any loop or closure runs at most once, and
    nothing written above it can print after it.  A confident match that breaks
    that order shows an earlier guess was wrong, so both are withdrawn."""
    high, setter = 0, None
    for idx, ol in enumerate(lines):
        if ol.conf != "confident" or ol.site is None or ol.kind in ("cont", "pretty"):
            continue
        s = prog.sites[ol.site]
        if not s.in_main or s.anywhere:
            continue
        if s.line < high:
            _downgrade(lines, idx, "out-of-order")
            if setter is not None and lines[setter].kind in ("specific", "in-order", "prefix"):
                _downgrade(lines, setter, "out-of-order")
            continue
        if s.once:
            high, setter = max([s.line] + ol.also), idx


def map_run(prog: Program, output: str) -> Mapped:
    texts = output.split("\n")
    if texts and texts[-1] == "":
        texts.pop()
    K = len(prog.headers)
    starts, unexpected, nxt = {}, [], 1
    for i, t in enumerate(texts):
        if not t.startswith("== "):
            continue
        hit = _header_start(prog, t.rstrip(), nxt)
        if hit is None:
            unexpected.append(i)
        else:
            starts[hit] = i
            nxt = hit + 1
    start_at = {i: k for k, i in starts.items()}
    lines, sec = [], 0
    for i, t in enumerate(texts):
        sec = start_at.get(i, sec)
        lines.append(OutLine(t, sec))

    consumed, cursor, n, i = set(), {}, len(lines), 0

    def is_header(text: str) -> bool:
        t = text.rstrip()
        return t.startswith("== ") and any(h.lines[0].regex.match(t) for h in prog.headers)

    def local(s: Site, sec: int) -> bool:
        return s.in_main and not s.anywhere and s.section == sec

    def panic_line(ol: OutLine, L: int) -> None:
        ol.src, ol.end, ol.conf, ol.kind = L, L, "confident", "panic"

    while i < n:
        ol = lines[i]
        if not ol.text.strip():
            i += 1
            continue
        if i in start_at:
            h = prog.headers[ol.sec - 1]
            _assign(ol, h, "confident", "header")
            j = i + 1
            for tm in h.lines[1:]:
                if j < n and tm.regex.match(lines[j].text.rstrip()):
                    _assign(lines[j], h, "confident", "cont")
                    j += 1
                else:
                    break
            consumed.add(h.id)
            cursor[ol.sec] = h.line
            i = j
            continue
        pm = PANIC_RE.match(ol.text)
        if pm and os.path.basename(pm.group(1)) == prog.path.name:
            L = int(pm.group(2))
            panic_line(ol, L)
            j = i + 1
            while j < n:   # the message may span lines (assert_eq! does)
                t = lines[j].text
                if t.startswith("note: "):
                    panic_line(lines[j], L)
                    j += 1
                    break
                if not t.strip() or j in start_at or any(
                        s.lines and not s.wildcard_only and s.id not in consumed
                        and _in_section(s, lines[j].sec) and _match_full(s, lines, j, is_header)
                        for s in prog.sites):
                    break
                panic_line(lines[j], L)
                j += 1
            i = j
            continue

        cands = []
        for s in prog.sites:
            if s.id in consumed or not s.lines or not _in_section(s, ol.sec):
                continue
            m = _match_full(s, lines, i, is_header)
            if m:
                cands.append((s, m))
        if cands:
            sec, c0 = ol.sec, cursor.get(ol.sec, 0)

            def dist(s: Site) -> int:
                if not local(s, sec):
                    return 0
                return s.line - c0 if s.line >= c0 else 100000 + c0 - s.line

            cands.sort(key=lambda sm: (sm[1][0], not sm[0].wildcard_only, local(sm[0], sec),
                                       sm[0].lit_total, -dist(sm[0])), reverse=True)
            best, (span, extra) = cands[0]
            others = [s for s, _ in cands[1:]]
            if not others:
                conf, kind = "confident", "unique"
            elif all(o.lit_total < best.lit_total for o in others):
                conf, kind = "confident", "specific"
            elif (len({s.pattern_key() for s, _ in cands}) == 1
                  and all(local(s, sec) for s, _ in cands)):
                best, (span, extra) = min(cands, key=lambda sm: dist(sm[0]))
                ok = best.once and not best.cond
                conf, kind = ("confident" if ok else "inferred"), "in-order"
            else:
                conf, kind = "inferred", "ambiguous"
            if conf == "confident" and best.wildcard_only and prog.has_raw:
                conf = "inferred"
            for j in range(i, i + span):
                _assign(lines[j], best, conf, kind if j == i else "cont")
            for j in range(i + span, i + span + extra):
                _assign(lines[j], best, conf, "pretty")
            if conf == "confident":   # a guess must not use up a site
                if best.once:
                    consumed.add(best.id)
                if local(best, sec):
                    cursor[sec] = best.line
            i += span + extra
            continue

        pcs = []
        for s in prog.sites:
            if s.id in consumed or s.frag is None or s.frag.lit_chars == 0 or not _in_section(s, ol.sec):
                continue
            m = s.frag.regex.match(ol.text)
            if m:
                pcs.append((s, m.end()))
        if pcs:
            pcs.sort(key=lambda se: se[0].frag.lit_chars, reverse=True)
            s, end = pcs[0]
            unique = len(pcs) == 1 or pcs[1][0].frag.lit_chars < s.frag.lit_chars
            conf = "confident" if unique and s.frag.lit_chars >= 6 else "inferred"
            also = _terminators(prog, s, ol.text[end:])
            _assign(ol, s, conf, "prefix", also)
            if conf == "confident":
                for x in prog.sites:
                    if x.once and (x is s or (x.line in also and x.in_main == s.in_main)):
                        consumed.add(x.id)
                if local(s, ol.sec):
                    cursor[ol.sec] = s.line
        i += 1
    _check_wildcards(prog, lines)
    _check_order(prog, lines)
    silent = [k for k in range(1, K + 1) if k not in starts]
    return Mapped(lines, starts, silent, unexpected)


# ---------------------------------------------------------------------------
# README tables and week metadata
# ---------------------------------------------------------------------------

def split_row(row: str) -> list:
    cells, cur, code, i = [], [], False, 0
    while i < len(row):
        ch = row[i]
        if ch == "\\" and i + 1 < len(row) and row[i + 1] == "|":
            cur.append("|")
            i += 2
            continue
        if ch == "`":
            code = not code
        if ch == "|" and not code:
            cells.append("".join(cur).strip())
            cur = []
        else:
            cur.append(ch)
        i += 1
    cells.append("".join(cur).strip())
    if cells and cells[0] == "":
        cells.pop(0)
    if cells and cells[-1] == "":
        cells.pop()
    return cells


def readme_tables(text: str) -> dict:
    """{first header cell: {key: [cells...]}} for every markdown table."""
    out, lines, i = {}, text.split("\n"), 0
    while i < len(lines):
        if (lines[i].startswith("|") and i + 1 < len(lines)
                and re.match(r"\|[\s:|-]+$", lines[i + 1].strip())):
            header = split_row(lines[i])
            rows, i = {}, i + 2
            while i < len(lines) and lines[i].startswith("|"):
                cells = split_row(lines[i])
                if cells:
                    key = cells[0].strip("`").strip()
                    key = key[:-3] if key.endswith(".rs") else key
                    rows[key] = cells
                i += 1
            if header:
                out.setdefault(header[0], rows)
        else:
            i += 1
    return out


def week_topic(week_dir: Path) -> str:
    slides = week_dir / "slides.html"
    if slides.exists():
        m = re.search(r"<title>(.*?)</title>", slides.read_text(encoding="utf-8"), re.S)
        if m:
            return html.unescape(re.split(r"\s+[—-]\s+CS 326", m.group(1).strip())[0])
    readme = week_dir / "README.md"
    if readme.exists():
        first = readme.read_text(encoding="utf-8").split("\n", 1)[0].lstrip("# ")
        return first.split(" — ", 1)[-1]
    return week_dir.name


# ---------------------------------------------------------------------------
# Rendering
# ---------------------------------------------------------------------------

HUES = ["#0072B2", "#E69F00", "#009E73", "#CC79A7", "#D55E00", "#56B4E9", "#7B5EA7", "#9C6B2F"]
GREY = "#6a737d"
OUT_BG = "#f6f8fa"
XREF_RE = re.compile(r"show-errors\.sh\s+(e\d{4})")
LOC_RE = re.compile(r"^(\s*)(-->|:::) (\S+?):(\d+):(\d+)")


def esc(s: str) -> str:
    return html.escape(s, quote=True)


def inline_md(s: str) -> str:
    out, pos = [], 0
    for m in re.finditer(r"`([^`]+)`|\*\*([^*]+)\*\*|\*([^*\s][^*]*)\*", s):
        out.append(esc(s[pos:m.start()]))
        if m.group(1) is not None:
            out.append(f"<code>{esc(m.group(1))}</code>")
        elif m.group(2) is not None:
            out.append(f"<strong>{esc(m.group(2))}</strong>")
        else:
            out.append(f"<em>{esc(m.group(3))}</em>")
        pos = m.end()
    out.append(esc(s[pos:]))
    return "".join(out)


def _rgb(h: str) -> tuple:
    return tuple(int(h[k:k + 2], 16) for k in (1, 3, 5))


def _hex(rgb: tuple) -> str:
    return "#" + "".join(f"{max(0, min(255, round(v))):02x}" for v in rgb)


def _mix(a: str, b: str, t: float) -> str:
    return _hex(tuple(x * t + y * (1 - t) for x, y in zip(_rgb(a), _rgb(b))))


def _lum(h: str) -> float:
    def ch(v: float) -> float:
        v /= 255
        return v / 12.92 if v <= 0.03928 else ((v + 0.055) / 1.055) ** 2.4
    r, g, b = (ch(v) for v in _rgb(h))
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def _contrast(a: str, b: str) -> float:
    la, lb = sorted((_lum(a), _lum(b)), reverse=True)
    return (la + 0.05) / (lb + 0.05)


def sec_style(k: int) -> str:
    hue = GREY if k == 0 else HUES[(k - 1) % len(HUES)]
    tint = _mix(hue, "#ffffff", 0.12)
    dark, t = hue, 1.0
    while _contrast(dark, OUT_BG) < 5.0 and t > 0.2:
        t -= 0.05
        dark = _mix(hue, "#000000", t)
    fg = "#ffffff" if _contrast("#ffffff", hue) >= _contrast("#111111", hue) else "#111111"
    return f"--s:{hue};--st:{tint};--sd:{dark};--sb:{fg}"


def char_classes(src: Source) -> list:
    cls = [None] * len(src.text)
    for s, e, c in src.toks:
        for k in range(s, e):
            cls[k] = c
        if c != "s":
            continue
        tok = src.text[s:e]
        if not RAW_STR_RE.match(tok):
            for m in re.finditer(r"\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F]+\}|.)", tok):
                for k in range(s + m.start(), s + m.end()):
                    cls[k] = "esc"
        for m in re.finditer(r"\{\{|\}\}|\{[^{}\"\n]*\}", tok):
            if m.group(0) not in ("{{", "}}"):
                for k in range(s + m.start(), s + m.end()):
                    cls[k] = "fmt"
    return cls


def mark_ranges(src: Source, diags: list) -> tuple:
    """Per-character mark ids from diagnostic spans, merging overlaps."""
    ranges = []
    for d in diags:
        lvl = "w" if d["level"] == "warning" else "e"
        for sp in d["spans"]:
            for L in range(sp["ls"], sp["le"] + 1):
                if L < 1 or L > len(src.lines):
                    continue
                text = src.lines[L - 1]
                c0 = sp["cs"] if L == sp["ls"] else len(text) - len(text.lstrip()) + 1
                c1 = sp["ce"] if L == sp["le"] else len(text) + 1
                c0, c1 = max(1, c0), min(len(text) + 1, c1)
                if c1 <= c0:
                    continue
                base = src.starts[L - 1]
                ranges.append([base + c0 - 1, base + c1 - 1, sp["primary"], sp["label"], lvl])
    ranges.sort()
    merged = []
    for r in ranges:
        if merged and r[0] < merged[-1][1]:
            m = merged[-1]
            m[1] = max(m[1], r[1])
            m[2] = m[2] or r[2]
            if r[3] and r[3] not in m[3]:
                m[3] = (m[3] + "; " + r[3]) if m[3] else r[3]
            if r[4] == "e":
                m[4] = "e"
        else:
            merged.append(list(r))
    marks = [None] * len(src.text) if merged else None
    for idx, (a, b, _, _, _) in enumerate(merged):
        for k in range(a, b):
            marks[k] = idx
    return marks, merged


def render_line(src: Source, L: int, cls: list, marks=None, info=None) -> str:
    a = src.starts[L - 1]
    b = a + len(src.lines[L - 1])
    out, k = [], a
    while k < b:
        mk = marks[k] if marks else None
        j = k
        while j < b and (marks[j] if marks else None) == mk:
            j += 1
        inner, x = [], k
        while x < j:
            c, y = cls[x], x
            while y < j and cls[y] == c:
                y += 1
            t = esc(src.text[x:y])
            inner.append(f'<span class="{c}">{t}</span>' if c else t)
            x = y
        s = "".join(inner)
        if mk is not None:
            _, _, primary, label, lvl = info[mk]
            title = f' title="{esc(label)}"' if label else ""
            s = f'<mark class="ds {lvl}{" p" if primary else ""}"{title}>{s}</mark>'
        out.append(s)
        k = j
    return "".join(out)


def indent_of(text: str) -> int:
    return len(text) - len(text.lstrip(" "))


def code_row(src: Source, L: int, inner: str, *, extra: str = "", attrs: str = "",
             badge=None, tag: str = "div", tail: str = "") -> str:
    ind = indent_of(src.lines[L - 1])
    style = f' style="--i:{ind}"' if ind else ""
    b = f'<span class="badge">{badge}</span>' if badge is not None else ""
    cls = "ln" + (" " + extra if extra else "")
    inner = inner or "<br>"   # an empty row still copies as a line
    return f'<{tag} class="{cls}" data-l="{L}"{attrs}{style}>{b}{inner}{tail}</{tag}>'


def linkify(text: str, known: dict) -> str:
    out, pos = [], 0
    for m in XREF_RE.finditer(text):
        target = known.get(m.group(1).lower())
        if target is None:
            continue
        out.append(esc(text[pos:m.start()]))
        out.append(f'<a class="xref" href="#{esc(target)}">{esc(m.group(0))}</a>')
        pos = m.end()
    out.append(esc(text[pos:]))
    return "".join(out)


def diag_rows(rendered: str, file_name: str, link_id: str) -> str:
    rows, lvl, in_help, here = [], "err", False, True
    for line in rendered.rstrip("\n").split("\n"):
        cls, src, body = "cont", None, None
        hm = re.match(r"(error|warning)(\[(E\d{4})\])?: ", line)
        nm = re.match(r"(note|help): ", line)
        lm = LOC_RE.match(line)
        if hm:
            lvl = "err" if hm.group(1) == "error" else "warn"
            in_help, cls = False, "hd"
            if hm.group(3):
                code = hm.group(3)
                body = (f'{hm.group(1)}[<a class="ecode" href="https://doc.rust-lang.org/error_codes/{code}.html"'
                        f' target="_blank" rel="noopener">{code}</a>]{esc(line[hm.end(2):])}')
        elif nm:
            in_help, cls = nm.group(1) == "help", "hd " + nm.group(1)
        elif lm:
            cls, here = "loc", os.path.basename(lm.group(3)) == file_name
            if here:
                src = int(lm.group(4))
                body = (f'{esc(lm.group(1))}{esc(lm.group(2))} <a href="#{esc(link_id)}/L{src}">'
                        f'{esc(lm.group(3))}:{lm.group(4)}:{lm.group(5)}</a>{esc(line[lm.end():])}')
        elif re.match(r"\s*\d+\s*\|", line):
            if in_help:
                cls = "sugg"
            else:
                cls = "src"
                if here:
                    src = int(re.match(r"\s*(\d+)", line).group(1))
        elif re.match(r"\s*\d+\s+[+~-]", line):
            cls = "sugg"
        elif re.match(r"\s*\|", line):
            cls = "gut"
            body = re.sub(r"(?<=[\s|_])(\^+)", r'<span class="u1">\1</span>', esc(line))
            body = re.sub(r"(?<=\s)(-+)(?=\s|$)", r'<span class="u2">\1</span>', body)
        elif re.match(r"\s*= ", line):
            cls = "eq"
        elif line.startswith("For more information"):
            cls = "more"
        attrs = f' data-src="{src}"' if src else ""
        ind = indent_of(line)
        style = f' style="--i:{ind}"' if ind else ""
        rows.append(f'<div class="dl {cls} {lvl}"{attrs}{style}>{body if body is not None else esc(line)}</div>')
    return "".join(rows)


def p95(vals: list) -> int:
    if not vals:
        return 0
    s = sorted(vals)
    return s[min(len(s) - 1, int(0.95 * len(s)))]


def split_left(code_lens: list, out_lens: list) -> int:
    c, o = p95(code_lens) + 8, p95(out_lens) + 5
    return max(40, min(65, round(100 * c / (c + o))))


@dataclass
class Ctx:
    week: str
    known: dict          # e0506 -> article id
    readme: dict


def out_row(i: int, ol: OutLine, known: dict, badge=None) -> str:
    cls, attrs = "ol", f' data-o="{i + 1}"'
    if ol.kind == "header":
        cls += " hd"
    if ol.conf == "confident" and ol.src:
        attrs += f' data-src="{ol.src}"'
        if ol.also:
            attrs += f' data-also="{" ".join(str(a) for a in ol.also)}"'
        if ol.owner:
            cls += " far"
            by = ol.owner + (f" ({ol.owner_note})" if ol.owner_note else "")
            attrs += f' data-by="{esc(by)}"'
            if ol.drop:
                attrs += ' data-drop="1"'
        if ol.kind == "panic":
            cls += " panic"
    ind = indent_of(ol.text)
    style = f' style="--i:{ind}"' if ind else ""
    b = f'<span class="badge">{badge}</span>' if badge is not None else ""
    return f'<div class="{cls}"{attrs}{style}>{b}{linkify(ol.text, known) or "<br>"}</div>'


def render_program(p: Program, cap: dict, mapped: list, ctx: Ctx) -> str:
    src = p.src
    cls = char_classes(src)
    linked = set()
    for mr in mapped:
        for ol in mr.lines:
            if ol.conf == "confident" and ol.src:
                linked.add(ol.src)
                linked.update(ol.also)
    site_line = {}
    for s in p.sites:
        if s.line in linked:
            for L in range(s.line, s.end_line + 1):
                site_line.setdefault(L, s.line)
    marks, info = mark_ranges(src, cap.get("diagnostics", []))

    def crow(L: int, **kw) -> str:
        extra, attrs = "", ""
        if L in site_line:
            extra, attrs = "ps", f' data-p="{site_line[L]}"'
        return code_row(src, L, render_line(src, L, cls, marks, info), extra=extra, attrs=attrs, **kw)

    runs = cap["runs"]
    cells = []
    pre = []
    first_code = 1
    if p.doc_end:
        more = p.doc_end - 1
        tail = f'<span class="more">▸ {more} more line{"s" if more != 1 else ""}</span>' if more else ""
        pre.append('<details class="doc"><summary>' + crow(1, tag="span", tail=tail) + "</summary>")
        pre += [crow(L) for L in range(2, p.doc_end + 1)]
        pre.append("</details>")
        first_code = p.doc_end + 1
    pre += [crow(L) for L in range(first_code, p.main_open + 1)]
    cells.append(f'<div class="cell code pre">{"".join(pre)}</div><div class="cell out pre"></div>')

    for k in sorted(p.ranges):
        lo, hi = p.ranges[k]
        per_run, any_out = [], False
        for r, mr in enumerate(mapped, 1):
            idxs = [i for i, ol in enumerate(mr.lines) if ol.sec == k]
            while idxs and not mr.lines[idxs[-1]].text.strip():
                idxs.pop()
            while k == 0 and idxs and not mr.lines[idxs[0]].text.strip():
                idxs.pop(0)
            any_out = any_out or bool(idxs)
            per_run.append((r, mr, idxs))
        if k == 0 and lo > hi and not any_out:
            continue
        banded = k > 0 or any_out
        attrs = f' data-sec="{k}" id="{esc(p.stem)}-s{k}" style="{sec_style(k)}"' if banded else ""
        sec_cls = " sec" if banded else ""
        rows = [crow(L, badge=k if banded and L == lo else None) for L in range(lo, hi + 1)]
        outs = []
        for r, mr, idxs in per_run:
            body = []
            if len(runs) > 1:
                body.append(f'<div class="run-label">$ {esc(runs[r - 1]["cmd"])}</div>')
            badge_done = False
            for i in idxs:
                ol = mr.lines[i]
                b = None
                if banded and not badge_done and ol.text.strip():
                    b, badge_done = k, True
                body.append(out_row(i, ol, ctx.known, badge=b))
            outs.append(f'<div class="run" data-run="{r}">{"".join(body)}</div>')
        cells.append(f'<div class="cell code{sec_cls}"{attrs}>{"".join(rows)}</div>'
                     f'<div class="cell out{sec_cls}"{attrs.replace(" id=", " data-id=")}>{"".join(outs)}</div>')

    post = [crow(L) for L in range(p.main_close, len(src.lines) + 1)]
    exits = []
    for r, run in enumerate(runs, 1):
        if run["timed_out"]:
            status = "timed out"
        elif run["exit"] is None:
            status = "did not run (build failed)"
        else:
            status = f"exit status {run['exit']}"
        exits.append(f'<div class="run" data-run="{r}"><div class="exit">{esc(status)}</div></div>')
    cells.append(f'<div class="cell code post">{"".join(post)}</div><div class="cell out post">{"".join(exits)}</div>')

    code_lens = [len(x) for x in src.lines]
    out_lens = [len(ol.text) for mr in mapped for ol in mr.lines]
    row = ctx.readme.get("Program", {}).get(p.stem)
    cap_html = ""
    if row and len(row) >= 3:
        cap_html = f'<p class="cap">{inline_md(row[1])} <span class="sep">·</span> <span class="point">point at {inline_md(row[2])}</span></p>'
    if len(runs) > 1:
        cmds = "".join(
            f'<button type="button" class="tab" data-run="{r}" aria-selected="{"true" if r == 1 else "false"}">'
            f'$ {esc(run["cmd"])}</button>' for r, run in enumerate(runs, 1))
    else:
        cmds = f'<span class="cmd">$ {esc(runs[0]["cmd"])}</span>'
    build = ""
    diags = cap.get("diagnostics", [])
    if diags:
        nw = sum(1 for d in diags if d["level"] == "warning")
        ne = len(diags) - nw
        what = ", ".join(x for x in (f"{ne} error{'s' if ne != 1 else ''}" if ne else "",
                                     f"{nw} warning{'s' if nw != 1 else ''}" if nw else "") if x)
        rows_html = "".join(diag_rows(d["rendered"], p.path.name, p.stem) for d in diags)
        build = (f'<details class="build" open><summary>cargo build: {what} <kbd>e</kbd></summary>'
                 f'<div class="diag">{rows_html}</div></details>')
    notes = []
    if re.search(r"\benv::args", src.mask):
        notes.append("argv")       # the Playground runs `cargo run` with no arguments
    if re.search(r"\btype_name\b", src.mask):
        notes.append("typename")   # the crate is named `playground` there
    if re.search(r"\{:p\}|as_ptr\(|\.capacity\(", src.mask):
        notes.append("nondet")     # addresses and capacities differ every run
    runs_data = [{"cmd": run["cmd"], "args": run["args"]} for run in runs]
    return (f'<article class="ex" id="{esc(p.stem)}" data-kind="program" data-run="1" tabindex="-1"'
            f' data-file="src/bin/{esc(p.stem)}.rs" data-notes="{" ".join(notes)}"'
            f' data-runs="{esc(json.dumps(runs_data, ensure_ascii=False))}"'
            f' style="--left:{split_left(code_lens, out_lens)}%">'
            f'<header class="exh"><h2><span class="num">{esc(p.num)}</span> {inline_md(p.title)}</h2>'
            f'{cap_html}<div class="cmds">{cmds}</div>{build}</header>'
            f'<div class="grid">{"".join(cells)}</div></article>')


def render_broken(b: Broken, cap: dict, ctx: Ctx, art_id: str) -> str:
    src = b.src
    cls = char_classes(src)
    marks, info = mark_ranges(src, cap["diagnostics"])
    rows = [code_row(src, L, render_line(src, L, cls, marks, info),
                     extra="hc" if L <= b.head_end else "") for L in range(1, len(src.lines) + 1)]
    why = []
    for para in b.why:
        prose, block = [], []
        for x in para:
            if re.match(r" {4,}", x):
                block.append(x.strip())
            else:
                if block:
                    prose.append(f'<code class="q">{esc(" ".join(block))}</code>')
                    block = []
                prose.append(inline_md(x.strip()))
        if block:
            prose.append(f'<code class="q">{esc(" ".join(block))}</code>')
        why.append(f"<p>{' '.join(prose)}</p>")
    nfix = len(b.fixes)
    if nfix:
        why.append('<ol class="fixes">' + "".join(f"<li>{inline_md(f)}</li>" for f in b.fixes) + "</ol>")
    if b.note:
        why.append(f'<p class="note">{inline_md(b.note)}</p>')
    label = "the fix" if nfix == 1 else f"{nfix} fixes" if nfix else "the explanation"
    card = (f'<details class="why"><summary>Why, and {label} <kbd>e</kbd></summary>'
            f'<div class="whyb">{"".join(why)}</div></details>')
    status = "timed out" if cap.get("timed_out") else f"exit status {cap['exit']}"
    out = (f'{card}<div class="cmdline">$ {esc(cap["cmd"])}</div>'
           f'<div class="diag">{diag_rows(cap["rendered"], b.path.name, art_id)}</div>'
           f'<div class="exit">{esc(status)}</div>')
    err = re.sub(r"^error\[E\d{4}\]:\s*", "", b.error)
    row = ctx.readme.get("File", {}).get(b.stem)
    cap_html = ""
    if row and len(row) >= 3:
        cap_html = f'<p class="cap">{inline_md(row[1])} <span class="sep">·</span> <span class="point">fix: {inline_md(row[2])}</span></p>'
    code_lens = [len(x) for x in src.lines]
    out_lens = [len(x) for x in cap["rendered"].split("\n")]
    return (f'<article class="ex broken" id="{esc(art_id)}" data-stem="{esc(b.stem)}" data-kind="broken"'
            f' tabindex="-1" style="--left:{split_left(code_lens, out_lens)}%">'
            f'<header class="exh"><h2><span class="num">{esc(b.code or b.stem)}</span> {inline_md(err)}</h2>'
            f'{cap_html}<div class="cmds"><span class="cmd">$ ./show-errors.sh {esc(b.short)}</span></div></header>'
            f'<div class="grid"><div class="cell code">{"".join(rows)}</div>'
            f'<div class="cell out">{out}</div></div></article>')


PAGE = Template("""<!DOCTYPE html>
<!-- Generated by tools/pointat.py from $week/examples/. Do not edit by hand:
     run `python3 tools/pointat.py $week` (or add --no-run to re-render). -->
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="generator" content="pointat">
<title>$title</title>
<script>document.documentElement.classList.add("js");window.POINTAT=$config;</script>
<style>
$css
</style>
</head>
<body>
<header class="bar">
  <span class="back">$backlink</span>
  <a class="home" href="#index">$bartitle</a>
  <nav class="picker" aria-label="examples">
    <button type="button" class="prev" title="previous example (p)">&lsaquo;</button>
    <select id="pick" aria-label="choose an example">$options</select>
    <button type="button" class="next" title="next example (n)">&rsaquo;</button>
  </nav>
  <span class="legend"><kbd>Space</kbd>/<kbd>j</kbd>/<kbd>k</kbd> step sections &middot; click a line to pin &middot; <kbd>n</kbd>/<kbd>p</kbd> example &middot; <kbd>i</kbd> edit and run</span>
  <span class="size"><button type="button" data-fs="-2" title="smaller (-)">A&minus;</button><button type="button" data-fs="2" title="larger (+)">A+</button></span>
</header>
<main>
$index
$articles
</main>
<footer class="foot">$footer</footer>
<script>
$js
</script>
</body>
</html>
""")

INCLASS_BACKLINK = '<a href="README.md">CS 326 &middot; Week {n} in class</a>'
SITE_BACKLINK = '<a href="./">&larr; In Class</a>'
SERVE_BACKLINK = '<a href="/">CS 326 &middot; served here</a>'
PLAYGROUND = "https://play.rust-lang.org/execute"


def render_index(week: str, topic: str, programs: list, brokens: list, ids: dict, ctx: Ctx,
                 mapped: dict) -> str:
    n = week[4:]
    prog_rows = []
    for p in programs:
        row = ctx.readme.get("Program", {}).get(p.stem) or []
        idea = inline_md(row[1]) if len(row) > 1 else inline_md(p.title)
        point = inline_md(row[2]) if len(row) > 2 else ""
        name = p.stem[len(p.num) + 1:] if p.stem.startswith(p.num + "_") else p.stem
        prog_rows.append(f'<tr><td class="num">{esc(p.num)}</td><td><a href="#{esc(p.stem)}">{esc(name)}</a></td>'
                         f"<td>{idea}</td><td>{point}</td><td class=\"num\">{len(p.headers)}</td></tr>")
    broken_rows = []
    for b in brokens:
        row = ctx.readme.get("File", {}).get(b.stem) or []
        fix = inline_md(row[2]) if len(row) > 2 else ""
        err = re.sub(r"^error\[E\d{4}\]:\s*", "", b.error)
        broken_rows.append(f'<tr><td class="num"><a href="#{esc(ids[b.stem])}">{esc(b.code or b.stem)}</a></td>'
                           f"<td>{inline_md(err)}</td><td>{fix}</td></tr>")
    return f"""<section id="index" class="index">
<h1>Week {esc(n)} &middot; {esc(topic)}</h1>
<p class="lede">Every program in <code>examples/src/bin/</code> beside the output it printed, and every
file in <code>examples/broken/</code> beside what <code>rustc</code> said about it. Each coloured,
numbered section of code sits in the same row as the output it produced. Click an output line to light
up the <code>println!</code> that printed it, or click a <code>println!</code> to find its output.
Press <kbd>i</kbd> on a program to edit it and run it. On a phone the two columns do not fit, so a
<b>Code</b>/<b>Output</b> switch shows one at a time.</p>
<h2>The programs</h2>
<table class="list"><thead><tr><th></th><th>Program</th><th>The one idea</th><th>The line to point at</th><th>Sections</th></tr></thead>
<tbody>{"".join(prog_rows)}</tbody></table>
<h2 id="broken">Programs that must not compile</h2>
<table class="list"><thead><tr><th>Error</th><th>What rustc says</th><th>The fix</th></tr></thead>
<tbody>{"".join(broken_rows)}</tbody></table>
<h2>Keys</h2>
<table class="keys"><tbody>
<tr><td><kbd>Space</kbd> <kbd>j</kbd> <kbd>&darr;</kbd> <kbd>&rarr;</kbd></td><td>next section (dims the rest)</td></tr>
<tr><td><kbd>Shift</kbd>+<kbd>Space</kbd> <kbd>k</kbd> <kbd>&uarr;</kbd> <kbd>&larr;</kbd></td><td>previous section</td></tr>
<tr><td><kbd>0</kbd>&ndash;<kbd>9</kbd></td><td>jump to that section</td></tr>
<tr><td>click</td><td>pin an output line and the code that printed it (click again to unpin)</td></tr>
<tr><td><kbd>Esc</kbd></td><td>unpin, then leave section focus</td></tr>
<tr><td><kbd>n</kbd> <kbd>p</kbd></td><td>next or previous example</td></tr>
<tr><td><kbd>r</kbd></td><td>next run, for programs run more than once</td></tr>
<tr><td><kbd>e</kbd></td><td>open or close the explanation (broken files) or the build warnings</td></tr>
<tr><td><kbd>+</kbd> <kbd>-</kbd></td><td>larger or smaller text (remembered)</td></tr>
<tr><td><kbd>i</kbd></td><td>edit this program and run it</td></tr>
<tr><td><kbd>&#8984;</kbd>/<kbd>Ctrl</kbd>+<kbd>&crarr;</kbd></td><td>run what you have edited</td></tr>
<tr><td><kbd>c</kbd></td><td>compare: what you changed in the code, and what changed in the
output (press <kbd>Esc</kbd> first if you are typing)</td></tr>
<tr><td><kbd>o</kbd></td><td>on a narrow screen, switch between the code and the output column</td></tr>
</tbody></table>
</section>"""


def crate_edition(week: str) -> str:
    toml = REPO / week / "examples" / "Cargo.toml"
    m = re.search(r'^\s*edition\s*=\s*"(\d{4})"', toml.read_text(encoding="utf-8"), re.M) \
        if toml.exists() else None
    return m.group(1) if m else "2021"


def render_page(week: str, programs: list, brokens: list, caps: dict, mapped: dict,
                backlink: str, cfg: dict | None = None) -> str:
    week_dir = REPO / week
    topic = week_topic(week_dir)
    readme_path = week_dir / "README.md"
    readme = readme_tables(readme_path.read_text(encoding="utf-8")) if readme_path.exists() else {}
    ids, seen = {}, set()
    for b in brokens:
        aid = b.short if b.short not in seen else b.stem
        seen.add(aid)
        ids[b.stem] = aid
    known = {b.short.lower(): ids[b.stem] for b in brokens if b.code}
    ctx = Ctx(week, known, readme)
    arts = [render_program(p, caps["programs"][p.stem], mapped[p.stem], ctx) for p in programs]
    arts += [render_broken(b, caps["broken"][b.stem], ctx, ids[b.stem]) for b in brokens]
    opts = ['<option value="index">All programs</option>']
    opts.append(f'<optgroup label="Programs ({len(programs)})">' + "".join(
        f'<option value="{esc(p.stem)}">{esc(p.num)} &middot; {esc(p.stem[len(p.num) + 1:] or p.stem)}</option>'
        for p in programs) + "</optgroup>")
    opts.append(f'<optgroup label="Must not compile ({len(brokens)})">' + "".join(
        f'<option value="{esc(ids[b.stem])}">{esc(b.code or b.stem)} &middot; {esc(b.stem[6:].replace("_", " "))}</option>'
        for b in brokens) + "</optgroup>")
    when = caps.get("captured_at", "")
    try:
        when = datetime.datetime.fromisoformat(when).strftime("%Y-%m-%d %H:%M")
    except ValueError:
        pass
    footer = (f"Captured {esc(when)} &middot; {esc(caps.get('rustc', '?'))} &middot; {esc(caps.get('os', '?'))}. "
              "Addresses and capacities differ from run to run and machine to machine; that is the point. "
              "Generated by <code>tools/pointat.py</code>.")
    n = week[4:]
    return PAGE.substitute(
        week=week,
        title=esc(f"Code and Output: {topic} — CS 326 Week {n}"),
        css=(HERE / "pointat.css").read_text(encoding="utf-8"),
        js=((HERE / "pointat.bands.js").read_text(encoding="utf-8")
            + (HERE / "pointat.js").read_text(encoding="utf-8")
            + (HERE / "pointat.edit.js").read_text(encoding="utf-8")),
        config=json.dumps({
            "tool": "pointat/2", "week": week, "runner": "playground", "endpoint": PLAYGROUND,
            "channel": "stable", "mode": "debug", "edition": crate_edition(week),
            "captured_rustc": caps.get("rustc", ""), "timeout": 15,
            "palette": [sec_style(k) for k in range(9)],
            **(cfg or {}),
        }, ensure_ascii=False),
        backlink=backlink,
        bartitle=f"CS 326 &middot; Week {esc(n)} &middot; Code and output",
        options="".join(opts),
        index=render_index(week, topic, programs, brokens, ids, ctx, mapped),
        articles="\n".join(arts),
        footer=footer,
    )


def load_week(week: str) -> tuple:
    ex = REPO / week / "examples"
    return ([load_program(f) for f in sorted((ex / "src" / "bin").glob("*.rs"))],
            [load_broken(f) for f in sorted((ex / "broken").glob("*.rs"))])


def build_page(week: str, backlink: str, cfg: dict | None = None) -> str:
    """A week's page, from the sources on disk and its committed captures."""
    programs, brokens = load_week(week)
    caps = json.loads(captures_path(week).read_text(encoding="utf-8"))
    mapped = {p.stem: [map_run(p, r["output"]) for r in caps["programs"][p.stem]["runs"]]
              for p in programs if p.stem in caps["programs"]}
    return render_page(week, programs, brokens, caps, mapped, backlink, cfg)


# ---------------------------------------------------------------------------
# Checks, dump, publish, main
# ---------------------------------------------------------------------------

def check_week(week: str, programs: list, brokens: list, caps: dict, mapped: dict) -> list:
    problems = []
    if not caps.get("build", {}).get("success", False):
        problems.append(f"{week}: cargo build failed\n{caps.get('build', {}).get('stderr', '')}")
    for p in programs:
        c = caps["programs"].get(p.stem)
        where = f"{week}/{p.stem}"
        if c is None:
            problems.append(f"{where}: no capture (run without --no-run)")
            continue
        if c["source_sha1"] != sha1(p.src.text):
            problems.append(f"{where}: source changed since it was captured")
        if [r["cmd"] for r in c["runs"]] != [run[0] for run in p.runs]:
            problems.append(f"{where}: Run: lines changed since it was captured")
        for d in c["diagnostics"]:
            if d["level"] == "error":
                problems.append(f"{where}: build error: {d['message']}")
        for run, mr in zip(c["runs"], mapped.get(p.stem, [])):
            label = f"{where} ({run['cmd']})"
            if run["timed_out"]:
                problems.append(f"{label}: timed out")
            elif run["exit"] != 0:
                problems.append(f"{label}: exit status {run['exit']}")
            if mr.silent:
                problems.append(f"{label}: headers never printed for sections {mr.silent}")
            for i in mr.unexpected:
                if mr.lines[i].conf == "none":
                    problems.append(f"{label}: output line {i + 1} {mr.lines[i].text!r} matches no header")
            if str(REPO) in run["output"]:
                problems.append(f"{label}: output contains the absolute checkout path")
    for b in brokens:
        c = caps["broken"].get(b.stem)
        where = f"{week}/broken/{b.path.name}"
        if c is None:
            problems.append(f"{where}: no capture")
            continue
        if c["source_sha1"] != sha1(b.src.text):
            problems.append(f"{where}: source changed since it was captured")
        if c.get("timed_out"):
            problems.append(f"{where}: rustc timed out")
        elif c["exit"] == 0:
            problems.append(f"{where}: compiled, but it is supposed not to")
        elif b.code and f"error[{b.code}]" not in c["rendered"]:
            codes = sorted(set(re.findall(r"error\[(E\d{4})\]", c["rendered"])))
            problems.append(f"{where}: expected {b.code}, rustc said {codes or 'no error code'}")
        if str(REPO) in c["rendered"]:
            problems.append(f"{where}: diagnostics contain the absolute checkout path")
    return problems


def dump_week(week: str, programs: list, brokens: list, mapped: dict) -> dict:
    out = {"week": week, "programs": [], "broken": []}
    for p in programs:
        out["programs"].append({
            "stem": p.stem, "main": [p.main_open, p.main_close],
            "sections": {str(k): list(v) for k, v in p.ranges.items()},
            "runs": [
                {"cmd": cmd, "silent": mr.silent, "unexpected": mr.unexpected,
                 "lines": [[i + 1, ol.sec, ol.text, ol.src, ol.conf, ol.kind, ol.also]
                           for i, ol in enumerate(mr.lines)]}
                for (cmd, _, _), mr in zip(p.runs, mapped[p.stem])],
            "sites": [{"line": s.line, "end": s.end_line, "macro": s.macro, "fmt": s.fmt, "once": s.once,
                       "cond": s.cond, "section": s.section, "header": s.header, "owner": s.owner}
                      for s in p.sites],
        })
    for b in brokens:
        out["broken"].append({"stem": b.stem, "code": b.code, "error": b.error, "why": b.why,
                              "fixes": b.fixes, "note": b.note})
    return out


EX_LINK_RE = re.compile(r' ?<a class="ex-link"[^>]*>.*?</a>|\n\n[ \t]*\.reveal a\.ex-link[^{]*\{[^}]*\}')


def normalize_deck(text: str) -> str:
    """A deck without the examples links and their CSS, for comparing copies."""
    return EX_LINK_RE.sub("", text)


def publish_week(site: Path, week: str, page: str) -> list:
    """Write the site's copies; return problems."""
    dest = site / "docs" / "inclass"
    if not dest.is_dir():
        return [f"--publish: {dest} is not a directory"]
    problems = []
    target = dest / f"{week}-examples.html"
    target.write_text(page, encoding="utf-8")
    log(f"{week}: wrote {target}")
    slides = REPO / week / "slides.html"
    if not slides.exists():
        return problems
    n = week[4:]
    text = slides.read_text(encoding="utf-8")
    new = text.replace(INCLASS_BACKLINK.format(n=n), SITE_BACKLINK)
    new = new.replace('href="examples.html#', f'href="{week}-examples.html#')
    deck = dest / f"{week}-slides.html"
    if deck.exists():
        old = deck.read_text(encoding="utf-8")
        if old == new:
            return problems
        if normalize_deck(old) != normalize_deck(new):
            import difflib
            diff = difflib.unified_diff(normalize_deck(old).splitlines(), normalize_deck(new).splitlines(),
                                        str(deck), str(slides), n=0, lineterm="")
            shown = "\n".join(list(diff)[:40])
            problems.append(f"--publish: {deck} differs from {slides} beyond the back-link and examples "
                            f"links; not replacing it. Reconcile these lines first:\n{shown}")
            return problems
    deck.write_text(new, encoding="utf-8")
    log(f"{week}: wrote {deck}")
    return problems


def normalize_week(arg: str) -> str:
    m = re.fullmatch(r"(?:\./)?(?:week)?0*(\d+)/?", arg)
    if not m:
        raise SystemExit(f"not a week: {arg!r} (expected e.g. week04)")
    return f"week{int(m.group(1)):02d}"


def main(argv=None) -> int:
    argv = sys.argv[1:] if argv is None else list(argv)
    if argv and argv[0] == "serve":
        sys.modules.setdefault("pointat", sys.modules[__name__])
        sys.path.insert(0, str(HERE))
        import pointat_serve
        return pointat_serve.serve_main(argv[1:])
    ap = argparse.ArgumentParser(prog="pointat", description=__doc__.split("\n\n")[0],
                                 epilog="See the module docstring (or tools/README.md) for details.")
    ap.add_argument("weeks", nargs="*", help="week directories, e.g. week04")
    ap.add_argument("--all", action="store_true", help="every weekNN/ with examples/Cargo.toml")
    ap.add_argument("--no-run", action="store_true", help="render from captures.json; build and run nothing")
    ap.add_argument("--check", action="store_true", help="exit 1 if any check fails")
    ap.add_argument("--dump", action="store_true", help="print the derived model as JSON on stdout")
    ap.add_argument("--publish", metavar="SITE", help="also write the course site's copies under SITE/docs/inclass/")
    ap.add_argument("--timeout", type=float, default=10.0, help="seconds per program run (default 10)")
    args = ap.parse_args(argv)
    if args.all == bool(args.weeks):
        ap.error("name the weeks, or pass --all")
    if args.all:
        weeks = sorted(d.name for d in REPO.glob("week[0-9]*") if (d / "examples" / "Cargo.toml").exists())
    else:
        weeks = [normalize_week(w) for w in args.weeks]

    all_problems, dumps, not_written = [], [], []
    for week in weeks:
        ex = REPO / week / "examples"
        if not (ex / "Cargo.toml").exists():
            all_problems.append(f"{week}: no examples/Cargo.toml")
            not_written.append(week)
            continue
        programs = [load_program(f) for f in sorted((ex / "src" / "bin").glob("*.rs"))]
        brokens = [load_broken(f) for f in sorted((ex / "broken").glob("*.rs"))]
        if args.no_run:
            path = captures_path(week)
            if not path.exists():
                all_problems.append(f"{week}: no captures at {path}; run without --no-run first")
                not_written.append(week)
                continue
            caps = json.loads(path.read_text(encoding="utf-8"))
        else:
            caps = save_captures(week, capture_week(week, programs, brokens, args.timeout))
        mapped = {}
        for p in programs:
            c = caps["programs"].get(p.stem)
            if c is None:
                continue
            mapped[p.stem] = [map_run(p, r["output"]) for r in c["runs"]]
            for r, mr in zip(c["runs"], mapped[p.stem]):
                body = [ol for ol in mr.lines if ol.text.strip()]
                linked = sum(1 for ol in body if ol.conf == "confident")
                status = "timed out" if r["timed_out"] else f"exit {r['exit']}"
                args_s = (" -- " + " ".join(r["args"])) if r["args"] else ""
                log(f"  {p.stem + args_s:<44} {status:<8} {len(mr.starts)}/{len(p.headers)} sections"
                    f"  {linked}/{len(body)} lines linked")
        missing = [p.stem for p in programs if p.stem not in caps["programs"]]
        missing += [b.stem for b in brokens if b.stem not in caps["broken"]]
        problems = check_week(week, programs, brokens, caps, mapped)
        for b in brokens:
            c = caps["broken"].get(b.stem)
            if c is not None:
                codes = sorted(set(re.findall(r"error\[(E\d{4})\]", c["rendered"])))
                log(f"  broken/{b.path.name:<42} exit {c['exit']}  {', '.join(codes)}")
        all_problems += problems
        if missing:
            all_problems.append(f"{week}: page not written; no capture for {', '.join(missing)}")
            not_written.append(week)
            continue
        page = render_page(week, programs, brokens, caps, mapped, INCLASS_BACKLINK.format(n=week[4:]))
        out = REPO / week / "examples.html"
        out.write_text(page, encoding="utf-8")
        log(f"{week}: wrote {out.relative_to(REPO)} ({len(page) // 1024} KB)")
        if args.publish:
            site_page = render_page(week, programs, brokens, caps, mapped, SITE_BACKLINK)
            all_problems += publish_week(Path(args.publish).resolve(), week, site_page)
        if args.dump:
            dumps.append(dump_week(week, programs, brokens, mapped))
    if args.dump:
        json.dump(dumps if len(dumps) != 1 else dumps[0], sys.stdout, indent=1, ensure_ascii=False)
        print()
    for prob in all_problems:
        log(f"  ! {prob}")
    if all_problems:
        log(f"{len(all_problems)} problem{'s' if len(all_problems) != 1 else ''}")
        failed = args.check or not_written or any(p.startswith("--publish") for p in all_problems)
        return 1 if failed else 0
    if args.check:
        log("all checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
