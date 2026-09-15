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
├── pointat.css       the page's styles, inlined into every examples.html
├── pointat.js        the page's behaviour, inlined too
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

Deep links work too: `examples.html#04_adapters_and_closures/s3` opens section 3,
`#12_argv_and_write_all/r2` the second run, and `#e0506/L17` pins line 17.
