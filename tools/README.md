# pointat — code beside the output it printed

`pointat` turns a week's in-class examples into one web page,
`weekNN/examples.html`. Each program's source sits on the left and the output
it printed sits on the right, one row per `== section ==`. The page replaces
the two-terminal-windows-and-an-arrow routine: click an output line and the
`println!` that printed it lights up, or press `Space` to walk the sections
while the rest of the program dims.

```
tools/
├── pointat.py        the generator (Python 3.9+, standard library only)
├── pointat_serve.py  the local runner behind `pointat serve`
├── pointat.css       the page's styles, inlined into every examples.html
├── pointat.js        the page's behaviour: routing, stepping, pinning
├── pointat.bands.js  where a program's `== section ==` headers are
├── pointat.edit.js   editing, running, and the fresh output pane
├── parity-bands.mjs  checks pointat.bands.js against Python's sections
├── test_pointat.py   unit tests, with fixtures shaped after the corpus
└── README.md         this file
```

## The weekly routine

From the repo root, after writing or changing a week's programs:

```bash
python3 tools/pointat.py week05 --check
```

That builds `week05/examples` with cargo, runs every `src/bin/*.rs` the way
its `//! Run:` lines say, compiles every `broken/*.rs` exactly as
`show-errors.sh` does, and writes `week05/examples.html`. The raw captures go
in `week05/examples/.pointat/captures.json`. Commit both, so the page opens
for students from a `git pull` with no toolchain.

`--check` exits non-zero if anything looks wrong. That covers a program that
fails to build, exits non-zero, or times out, and output that is missing a
`== ` header its source prints. It also covers a broken file that compiles, or
whose errors lack the code in its name. The name check is what catches a new
rustc changing an error code.

```bash
python3 tools/pointat.py --all --check        # every week
python3 tools/pointat.py week05 --no-run      # re-render from captures.json (after CSS/JS edits)
python3 tools/pointat.py week05 --dump        # the derived mapping as JSON, for debugging
python3 -m unittest tools/test_pointat.py     # the tests
```

Addresses, capacities and a few offsets change on every run. That churn in
`captures.json` is expected, and the page footer says so to students. Use
`--no-run` while styling the page so captures stay put.

## Conventions the programs follow

The page is only as good as these, and every existing program follows them:

- The `//!` header's first line is `NN — the one idea`; its last lines are
  `//! Run:  cargo run --bin <stem>`, one per way to run it (for example
  `-- one two three` for arguments).
- `main` prints `println!("== title ==")` for the first section and
  `println!("\n== title ==")` for the rest, directly in `main`, not in a
  loop or helper.
- A `// fold: …` line directly above an item outside `main` (doc comments,
  attributes and blank lines may sit between) makes the page show that item
  folded to the comment line, the way the `//!` header is. Use it for code
  students should not read by accident, such as an exercise's answer. Put it
  anywhere else and the generator stops with the line number.
- Broken files start with a `//` block of at most 12 lines: the error line,
  a blank `//`, the explanation, a blank `//`, then `// FIX 1:` … lines.
- The week's `README.md` tables `| Program | The one idea | The line to point at |`
  and `| File | Error | Fix … |` provide the captions (optional).

## How the mapping decides

Sections are exact. The k-th header `println!` in `main` starts code section
k, and the k-th `== ` output line starts output section k. Code before the
first header, together with anything it printed, is section 0.

Line links are drawn only when they cannot be wrong. Each format string
becomes a regex, with `{}` matching anything, `{:p}` matching an address,
and `{{`/`}}` as literal braces. An output line is linked when:

- exactly one print site in that section, or outside `main` such as a helper
  or a `Drop` impl, matches it;
- or one match has clearly more literal text than the others;
- or identical once-only lines are matched in source order;
- or a `print!` fragment starts the line, in which case the rest of the line
  links to the `print!`/`println!` calls that finished it;
- or the line continues a `{:#?}` value, or is the message of a caught
  `panic!`.

Lines that `write_all` or `putc` produced are left unlinked, and so is any
line with two plausible sources. `--dump` shows every decision, including
the unrendered "inferred" ones.

## Publishing to the course site

```bash
python3 tools/pointat.py --all --no-run --publish ../USF-CS326-F26.github.io
cd ../USF-CS326-F26.github.io
python3 utils/gen_nav.py && python3 utils/gen_schedule.py
mkdocs build --strict && python3 utils/check_links.py
```

`--publish` writes `docs/inclass/weekNN-examples.html` and refreshes
`docs/inclass/weekNN-slides.html` from `weekNN/slides.html`. Both copies get
the site's back-link, and the slides' examples links get the site's file
names. It will not overwrite a published deck that differs from the source
in any other way. It prints the differing lines instead, so a fix made on the
site side is never lost.

## Editing and running in the page

Press <kbd>i</kbd> (or the **Edit and run** button) on any program. The captured
grid is replaced by an editor on the left and a fresh output pane on the right.
Change a line, press <kbd>&#8984;</kbd>/<kbd>Ctrl</kbd>+<kbd>Enter</kbd>, and
the program is compiled and run for real.

- **Section bands, not line links.** The editor's gutter keeps the coloured,
  numbered section stripes and the fresh output is grouped into the same
  sections, so the two sides still line up. Per-line links are not drawn for a
  fresh run: they are only as good as the mapping, and the mapping is only
  certain about the captured output. Revert brings them back.
- **Compare** (<kbd>c</kbd>) shows two diffs, both committed on the left and
  yours on the right. First **the code you changed**: the differing lines with
  three lines of context, numbered, with the unchanged stretches collapsed to
  a count. Then **what it printed**: the committed output beside your run's,
  one row per section, with the lines that differ marked. Blank lines are
  ignored, and it says so when the two runs came from different compilers or
  when the program prints addresses, which differ on every run anyway.
  **Revert** puts the original back, code and output both.
- The keys that are single letters only work when the editor does not have
  focus, because everything else you type belongs in the editor. <kbd>Esc</kbd>
  leaves the editor, and the buttons always work.
- Leaving the program and coming back shows the committed page again, with a
  line offering the unsaved edit. So a deck link never lands on somebody's
  half-finished experiment by surprise.
- A fresh run is always labelled with the runner and the compiler that
  produced it, so it cannot be mistaken for the capture.

Where it runs depends on how the page was opened:

| Opened from | Runs on | Notes |
|---|---|---|
| the course site, or a local file | play.rust-lang.org | no program arguments, stdout and stderr arrive separately, rustc is the Playground's stable |
| `python3 tools/pointat.py serve` | this machine | offline, arguments work, one merged stream, real diagnostic spans |

The classroom Wi-Fi allows play.rust-lang.org, and the course already tells
students the Playground is fine to use in a session. Pressing Run sends the
code there; nothing else leaves the machine.

## The local runner

```bash
python3 tools/pointat.py serve                 # every week, on 127.0.0.1:8326
python3 tools/pointat.py serve week04 --port 9000 --timeout 5 --open
```

It prints a URL per week and serves the generated pages with the local runner
wired in. Students can run it too, though they do not need to: every student
has rustup, and Python 3 ships with macOS and Ubuntu.

It compiles and runs code somebody typed into a browser page, on the machine
that runs it. It does not sandbox that code, and it is not meant to: it is your
machine and your code, the same as `cargo run`. What it does prevent is any
*other* page, user or machine making it run code:

- it binds 127.0.0.1, so nothing off the machine can reach it;
- only `POST /run` compiles anything, and it needs a token that exists only
  inside the pages that process served. A custom header is what makes every
  cross-origin attempt non-simple, and the server answers no `OPTIONS` and
  sends no CORS header, so a browser refuses such a request before it arrives;
- the `Host` header must name loopback and the right port, which is what stops
  DNS rebinding from making a hostile site same-origin and reading the token;
- an `Origin`, when sent, must be the server itself, and `null` is refused.

Each run gets a fresh temporary directory, one run happens at a time, output is
capped at 256 KiB while the program is still running, and a program that
overruns its timeout is killed by process group, so nothing it spawned outlives
it. `fn main() { loop { println!("x"); } }` is a safe thing to try in class.

## Keeping the page's section scan honest

The page infers one thing about edited code: where the `== section ==` headers
are. That lives in `tools/pointat.bands.js`, and it must agree with Python's
real lexer:

```bash
node tools/parity-bands.mjs          # 34 programs, 35 runs, 0 disagreements
```

Run it after touching either the scanner or the mapping in `pointat.py`. It
needs the committed captures, but no network and no Rust toolchain.

## Phones and tablets

Below 1000 pixels the two columns do not fit side by side, so each example gets
a **Code**/**Output** switch, pinned under the top bar, that shows one column at
a time. Switching keeps the section you were looking at, so the output for the
code on screen is one tap away. <kbd>o</kbd> does the same from a keyboard.

## Keys on the page

| Key | Does |
|---|---|
| `Space` `j` `↓` `→` | next section (the rest dims) |
| `Shift`+`Space` `k` `↑` `←` | previous section |
| `0`–`9` | that section |
| click | pin an output line and the code that printed it |
| `Esc` | unpin, then leave section focus |
| `n` `p` | next or previous example |
| `r` | next run (programs with more than one `Run:` line) |
| `e` | open or close the explanation or the build warnings |
| `+` `-` | text size (remembered) |
| `i` | edit this program and run it |
| `⌘`/`Ctrl`+`Enter` | run what you have edited |
| `c` | compare your run with the committed one, line by line |
| `o` | on a narrow screen, switch between the code and the output column |

Deep links work too: `examples.html#04_adapters_and_closures/s3` opens section 3,
`#12_argv_and_write_all/r2` the second run, and `#e0506/L17` pins line 17.
