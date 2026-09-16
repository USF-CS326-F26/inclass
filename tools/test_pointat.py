#!/usr/bin/env python3
"""Tests for tools/pointat.py.  Fixtures are shaped after real lines in the
weekNN/examples corpus; the file:line each one imitates is in its comment.

    python3 -m unittest tools/test_pointat.py
"""

import re
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import pointat as pa  # noqa: E402


def program(body: str, name: str = "t.rs"):
    return pa.load_program(Path(name), body)


def classes(src: str) -> list:
    return [(src[s:e], c) for s, e, c in pa.lex(src)]


class LexTest(unittest.TestCase):
    def test_comment_marker_inside_string_is_string(self):
        # week04/.../06_generics_and_bounds.rs:110, 02_iter_mut_and_enumerate.rs:63
        src = 'fn main() {\n    println!("a // b {x}");\n}\n'
        self.assertIn(('"a // b {x}"', "s"), classes(src))
        self.assertEqual(program(src).main_close, 3)

    def test_block_comment_inside_string(self):
        # week03/.../08_enums.rs:61
        src = 'fn main() {\n    let s = "/* no */ {";\n    println!("== x ==");\n}\n'
        p = program(src)
        self.assertEqual((p.main_open, p.main_close), (1, 4))
        self.assertEqual(len(p.headers), 1)

    def test_quotes_in_trailing_comment(self):
        # week02/.../07_borrow_shared.rs:21
        src = 'fn main() {\n    let r = &s; // & = "borrow, please"\n    println!("== a ==");\n}\n'
        self.assertEqual(len(program(src).headers), 1)

    def test_numbers_and_lifetimes(self):
        # week02/.../01_scalars.rs:26, week03/.../04_newtype.rs:55
        toks = classes("let a = 0b1_0000_0000_0000; let b = 0o7; let c = 4096usize; fn f<'a>(x: &'a str) {}")
        self.assertIn(("0b1_0000_0000_0000", "n"), toks)
        self.assertIn(("0o7", "n"), toks)
        self.assertIn(("4096usize", "n"), toks)
        self.assertIn(("'a", "lt"), toks)

    def test_char_literals(self):
        toks = classes(r"let a = '{'; let b = '\''; let c = b'\n';")
        self.assertIn(("'{'", "ch"), toks)
        self.assertIn((r"'\''", "ch"), toks)
        self.assertIn((r"b'\n'", "ch"), toks)


class TemplateTest(unittest.TestCase):
    def match(self, fmt: str, line: str) -> bool:
        return pa.build_tmpl(fmt).regex.match(line.rstrip()) is not None

    def test_escaped_braces(self):
        # week03/.../03_derive_and_copy.rs:20
        self.assertTrue(self.match("== Debug: what {{:?}} and assert_eq! print ==",
                                   "== Debug: what {:?} and assert_eq! print =="))

    def test_specs(self):
        self.assertTrue(self.match("load({name:>10}) -> {:?}", "load(      init) -> Ok(Image { size: 128 })"))
        self.assertTrue(self.match("  at {:p}", "  at 0x16bbd5eb8"))
        self.assertFalse(self.match("  at {:p}", "  at nowhere"))
        self.assertTrue(self.match("  kfree   {:#x}", "  kfree   0x80001000"))
        self.assertTrue(self.match("{table:#x?}", "["))

    def test_counts(self):
        t = pa.build_tmpl("  {:<38} {label}")
        self.assertEqual((t.lit_chars, t.nph), (0, 2))
        t = pa.build_tmpl("t == a  -> {}")
        self.assertEqual(t.lit_chars, 6)

    def test_unescape(self):
        self.assertEqual(pa.unescape(r'"\n== a ==\t\"q\" \\n \u{e9}"'), '\n== a ==\t"q" \\n é')
        self.assertEqual(pa.unescape('r#"raw "x" \\n"#'), 'raw "x" \\n')


class StructureTest(unittest.TestCase):
    def test_headers_and_sections(self):
        src = (
            "//! 99 — Title here.\n"          # 1
            "//!\n"                           # 2
            "//! Run:  cargo run --bin t\n"   # 3
            "fn main() {\n"                   # 4
            '    println!("pre {}", 1);\n'    # 5
            '    println!("== one ==");\n'    # 6
            "    // about two\n"              # 7
            '    println!("\\n== run(args) -> ");\n'  # 8  unclosed, trailing space
            "    for i in 0..2 {\n"           # 9
            '        println!("== not a header {i}");\n'  # 10
            "    }\n"                         # 11
            "}\n"                             # 12
        )
        p = program(src)
        self.assertEqual((p.num, p.title), ("99", "Title here."))
        self.assertEqual([h.line for h in p.headers], [6, 8])
        self.assertEqual(p.ranges, {0: (5, 5), 1: (6, 6), 2: (7, 11)})
        m = pa.map_run(p, "pre 1\n== one ==\n\n== run(args) -> \n== not a header 0\n== not a header 1\n")
        self.assertEqual(m.starts, {1: 1, 2: 3})
        self.assertEqual(m.silent, [])
        self.assertEqual([ol.sec for ol in m.lines], [0, 1, 1, 2, 2, 2])
        self.assertEqual([(ol.src, ol.conf) for ol in m.lines if ol.text.startswith("== not")],
                         [(10, "confident")] * 2)

    def test_fold_covers_the_item_under_it(self):
        # week04/.../12_argv_and_write_all.rs:79-90
        src = (
            "//! 99 — T.\n"                   # 1
            "//! Run:  cargo run --bin t\n"   # 2
            "struct A;\n"                     # 3
            "// fold: the answer\n"           # 4
            "\n"                              # 5
            "/// doc\n"                       # 6
            "#[inline]\n"                     # 7
            "fn run() -> i32 {\n"             # 8
            "    0\n"                         # 9
            "}\n"                             # 10
            "fn main() {\n"                   # 11
            '    println!("== a ==");\n'      # 12
            "    let _ = run();\n"            # 13
            "}\n"                             # 14
        )
        p = program(src)
        self.assertEqual(p.folds, [(4, 10)])
        cap = {"runs": [{"cmd": "cargo run --bin t", "args": [], "output": "== a ==\n",
                         "exit": 0, "timed_out": False}]}
        ctx = pa.Ctx("week99", {}, {})
        page = pa.render_program(p, cap, [pa.map_run(p, "== a ==\n")], ctx)
        self.assertIn('<details class="doc fold"><summary><span class="ln" data-l="1"', page)
        self.assertIn('<details class="fold"><summary><span class="ln" data-l="4"', page)
        self.assertIn("▸ 6 more lines", page)
        body = page[page.index('data-l="4"'):page.index("</details>", page.index('data-l="4"'))]
        self.assertEqual(re.findall(r'data-l="(\d+)"', body), [str(L) for L in range(4, 11)])
        self.assertEqual(page.count("<details"), page.count("</details>"))

    def test_fold_must_sit_on_an_item_outside_main(self):
        inside = 'fn main() {\n    // fold: no\n    fn f() {}\n    println!("== a ==");\n}\n'
        gap = "// fold: no\nconst N: u8 = 1;\nfn f() {}\nfn main() {}\n"
        on_main = "// fold: no\nfn main() {}\n"
        twice = "// fold: a\n// fold: b\nfn f() {}\nfn main() {}\n"
        for src in (inside, gap, on_main, twice):
            with self.subTest(src=src), self.assertRaises(SystemExit):
                program(src)
        in_string = 'const S: &str = "\n// fold: text, not a marker\n";\nfn main() {}\n'
        self.assertEqual(program(in_string).folds, [])

    def test_silent_sections_after_exit(self):
        src = 'fn main() {\n    println!("== a ==");\n    std::process::exit(0);\n    println!("\\n== b ==");\n}\n'
        m = pa.map_run(program(src), "== a ==\n")
        self.assertEqual(m.silent, [2])

    def test_loop_head_with_struct_literal(self):
        # week03/.../08_enums.rs:80-82
        src = (
            "fn main() {\n"
            '    println!("== t ==");\n'
            "    for t in [Trap::Timer, Trap::PageFault { addr: 0x8000_1000 },\n"
            "              Trap::Illegal] {\n"
            '        println!("  {:<38} kernel-handled: {}", format!("{t:?}"), t.ok());\n'
            "    }\n"
            "}\n"
        )
        p = program(src)
        site = next(s for s in p.sites if s.line == 5)
        self.assertFalse(site.once)
        out = "== t ==\n  Timer                                  kernel-handled: true\n" \
              "  PageFault { addr: 2147487744 }         kernel-handled: false\n" \
              "  Illegal                                kernel-handled: false\n"
        m = pa.map_run(p, out)
        self.assertEqual([(ol.src, ol.conf) for ol in m.lines[1:]], [(5, "confident")] * 3)

    def test_identical_templates_in_order(self):
        # week03/.../08_enums.rs:43-45
        src = (
            "fn main() {\n"
            '    println!("== show ==");\n'
            '    println!("{}", show(fault));\n'
            '    println!("{}", show(call));\n'
            '    println!("{}", show(tick));\n'
            "}\n"
        )
        m = pa.map_run(program(src), "== show ==\nPageFault\nSyscall(63)\nTimer\n")
        self.assertEqual([(ol.src, ol.conf) for ol in m.lines[1:]],
                         [(3, "confident"), (4, "confident"), (5, "confident")])

    def test_wildcard_is_not_confident_when_raw_writes_exist(self):
        # week04/.../12_argv_and_write_all.rs: `hello world` comes from write_all, not a println!
        src = ('fn main() {\n    println!("== a ==");\n    let _ = std::io::stdout().write_all(b"raw\\n");\n'
               '    println!("{}", s);\n}\n')
        m = pa.map_run(program(src), "== a ==\nraw\nhello\n")
        self.assertEqual([ol.conf == "confident" for ol in m.lines], [True, False, False])

    def test_print_prefix_and_terminators(self):
        # week04/.../12_argv_and_write_all.rs:131-139
        src = (
            "fn main() {\n"                                                           # 1
            '    println!("== console ==");\n'                                        # 2
            '    print!("bare write(STDOUT, b\\"hello world\\\\n\\") -> ");\n'        # 3
            '    let n = write(STDOUT, b"hello world\\n");\n'                         # 4
            '    println!("   <- returned {n:?}: three bytes went");\n'               # 5
            '    print!("write_all(STDOUT, b\\"hello world\\\\n\\")  -> ");\n'        # 6
            '    let r = write_all(STDOUT, b"hello world\\n");\n'                     # 7
            '    println!("   <- {r:?} after {} calls to write", 4);\n'               # 8
            "    std::io::stdout().write_all(b\"\");\n"                               # 9
            "}\n"
        )
        out = ('== console ==\n'
               'bare write(STDOUT, b"hello world\\n") -> hel   <- returned Ok(3): three bytes went\n'
               'write_all(STDOUT, b"hello world\\n")  -> hello world\n'
               '   <- Ok(()) after 4 calls to write\n')
        m = pa.map_run(program(src), out)
        self.assertEqual([(ol.src, ol.kind, ol.also, ol.conf) for ol in m.lines[1:]],
                         [(3, "prefix", [5], "confident"), (6, "prefix", [], "confident"),
                          (8, "unique", [], "confident")])

    def test_print_fragments_closed_by_bare_println(self):
        # week04/.../06_generics_and_bounds.rs:124-128
        src = (
            "fn main() {\n"
            '    println!("== live ==");\n'
            '    print!("live(&table) yields:");\n'
            "    for x in live(&table) {\n"
            '        print!(" {}", x);\n'
            "    }\n"
            "    println!();\n"
            "}\n"
        )
        m = pa.map_run(program(src), "== live ==\nlive(&table) yields: 7 9 2\n")
        self.assertEqual((m.lines[1].src, m.lines[1].also), (3, [5, 7]))

    def test_pretty_debug_continuation(self):
        # week03/.../03_derive_and_copy.rs:23-24
        src = ('fn main() {\n    println!("== d ==");\n    println!("{{:#?}} ->");\n'
               '    println!("{t:#?}");\n    println!("after");\n}\n')
        out = "== d ==\n{:#?} ->\nTicks {\n    count: 42,\n    hz: 10000000,\n}\nafter\n"
        m = pa.map_run(program(src), out)
        self.assertEqual([ol.src for ol in m.lines], [2, 3, 4, 4, 4, 4, 5])
        self.assertTrue(all(ol.conf == "confident" for ol in m.lines))

    def test_panic_block(self):
        # week02/.../06_drop.rs:69
        src = 'fn main() {\n    println!("== p ==");\n    let _ = std::panic::catch_unwind(|| {\n' \
              '        panic!("simulated fault");\n    });\n    println!("after");\n}\n'
        out = ("== p ==\nthread 'main' panicked at src/bin/06_drop.rs:4:9:\nsimulated fault\n"
               "note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\nafter\n")
        m = pa.map_run(program(src, "06_drop.rs"), out)
        self.assertEqual([ol.src for ol in m.lines], [2, 4, 4, 4, 6])

    def test_helper_and_drop_owner(self):
        # week03/.../07_drop_guard.rs:87-98
        src = (
            "fn main() {\n"
            '    println!("== d ==");\n'
            '    { let _a = Tracked::new("first"); }\n'
            "}\n"
            "impl Tracked {\n"
            "    pub fn new(name: &'static str) -> Tracked {\n"
            '        println!("  new  {name}");\n'
            "        Tracked { name }\n"
            "    }\n"
            "}\n"
            "impl Drop for Tracked {\n"
            "    fn drop(&mut self) {\n"
            '        println!("  drop {}", self.name);\n'
            "    }\n"
            "}\n"
        )
        m = pa.map_run(program(src), "== d ==\n  new  first\n  drop first\n")
        self.assertEqual([(ol.src, ol.owner, ol.drop) for ol in m.lines[1:]],
                         [(7, "Tracked::new()", False), (13, "Tracked::drop()", True)])

    def test_run_lines(self):
        # week04/.../12_argv_and_write_all.rs:18-19
        doc = ["//! Run:  cargo run --bin 12_argv_and_write_all",
               "//!       cargo run --bin 12_argv_and_write_all -- one two three"]
        self.assertEqual(pa.parse_runs(doc, "12_argv_and_write_all"),
                         [("cargo run --bin 12_argv_and_write_all", [], {}),
                          ("cargo run --bin 12_argv_and_write_all -- one two three", ["one", "two", "three"], {})])

    def test_run_lines_with_flags_env_and_pipes(self):
        doc = ["//! Run:  cargo run --bin t --release -- a b | head -3",
               "//!       N=3 cargo run --release --bin t -- x > out.txt",
               "//!       cargo run --bin other -- nope"]
        self.assertEqual([(a, e) for _, a, e in pa.parse_runs(doc, "t")],
                         [(["a", "b"], {}), (["x"], {"N": "3"})])


class BrokenTest(unittest.TestCase):
    def test_wrapped_error_fix_without_number_and_note(self):
        # week04/.../e0596_iter_mut_behind_shared_ref.rs:1-2, week02/.../e0505:6, e0506:8-12
        src = (
            "// error[E0596]: cannot borrow `*table` as mutable, as it is behind a `&`\n"
            "//               reference\n"
            "//\n"
            "// The loop is fine. The PARAMETER is wrong.\n"
            "//     error: quoted rustc line\n"
            "//\n"
            "// FIX 1: `for (i, slot) in table.iter_mut().enumerate()`\n"
            "//        -- one &mut per slot.\n"
            "// FIX: `for i in 0..table.len()` -- no iterator.\n"
            "// (A `return` right after the write is accepted: the borrow\n"
            "// checker can see the iterator is never used again.)\n"
            "fn main() {}\n"
        )
        b = pa.load_broken(Path("e0596_iter_mut_behind_shared_ref.rs"), src)
        self.assertEqual((b.code, b.short, b.head_end), ("E0596", "e0596", 11))
        self.assertEqual(b.error, "error[E0596]: cannot borrow `*table` as mutable, as it is behind a `&` reference")
        self.assertEqual(b.why, [["The loop is fine. The PARAMETER is wrong.", "    error: quoted rustc line"]])
        self.assertEqual(b.fixes, ["`for (i, slot) in table.iter_mut().enumerate()` -- one &mut per slot.",
                                   "`for i in 0..table.len()` -- no iterator."])
        self.assertTrue(b.note.startswith("(A `return`"))

    def test_spans_drop_suggestions_and_zero_width(self):
        # week03/.../e0004_non_exhaustive.rs, week02/.../e0106_dangling_reference.rs
        msg = {"level": "error", "code": {"code": "E0004"}, "message": "m", "rendered": "r",
               "spans": [{"file_name": "e0004.rs", "line_start": 12, "line_end": 12, "column_start": 11,
                          "column_end": 15, "is_primary": True, "label": None, "suggested_replacement": None}],
               "children": [{"spans": [
                   {"file_name": "e0004.rs", "line_start": 25, "line_end": 25, "column_start": 30,
                    "column_end": 30, "is_primary": True, "label": None, "suggested_replacement": ",\n x"},
                   {"file_name": "e0004.rs", "line_start": 3, "line_end": 3, "column_start": 6,
                    "column_end": 6, "is_primary": False, "label": "", "suggested_replacement": None},
                   {"file_name": "e0004.rs", "line_start": 3, "line_end": 3, "column_start": 6,
                    "column_end": 10, "is_primary": False, "label": "", "suggested_replacement": None}]}]}
        d = pa.flatten_diag(msg, "e0004.rs")
        self.assertEqual([(s["ls"], s["cs"], s["ce"], s["primary"]) for s in d["spans"]],
                         [(12, 11, 15, True), (3, 6, 10, False)])

    def test_overlapping_marks_merge(self):
        # week04/.../e0506_assign_while_iterating.rs:15 (cols 22-27 and 22-46)
        src = pa.Source(Path("x.rs"), "fn f() {\n    for (i, slot) in table.iter().enumerate() {\n}\n")
        diag = {"level": "error", "spans": [
            {"ls": 2, "le": 2, "cs": 22, "ce": 27, "primary": False, "label": "borrowed here"},
            {"ls": 2, "le": 2, "cs": 22, "ce": 46, "primary": False, "label": "later used here"}]}
        marks, info = pa.mark_ranges(src, [diag])
        self.assertEqual(len(info), 1)
        self.assertEqual(info[0][3], "borrowed here; later used here")


class ReadmeTest(unittest.TestCase):
    def test_table_with_pipes_in_code(self):
        text = ("| Program | The one idea | The line to point at |\n|---|---|---|\n"
                "| `04_adapters_and_closures` | three words | `.filter(|r| r.is_some())` |\n\n"
                "| File | Error | Fix |\n|---|---|---|\n| `e0506_assign_while_iterating.rs` | x | y |\n")
        t = pa.readme_tables(text)
        self.assertEqual(t["Program"]["04_adapters_and_closures"][2], "`.filter(|r| r.is_some())`")
        self.assertEqual(t["File"]["e0506_assign_while_iterating"][1], "x")


class HardeningTest(unittest.TestCase):
    """Inputs that once produced a confident link to the wrong line."""

    def test_braceless_closure_is_not_once_and_prints_anywhere(self):
        src = (
            "fn main() {\n"                                     # 1
            '    println!("== a ==");\n'                         # 2
            '    let show = |x: u32| println!("  v = {x}");\n'   # 3
            '    println!("\\n== b ==");\n'                      # 4
            "    show(1);\n"                                      # 5
            '    println!("  v = {}", 2);\n'                      # 6
            "}\n"
        )
        p = program(src)
        site = next(s for s in p.sites if s.line == 3)
        self.assertTrue(site.anywhere)
        self.assertFalse(site.once)
        m = pa.map_run(p, "== a ==\n\n== b ==\n  v = 1\n  v = 2\n")
        self.assertEqual([ol.conf == "confident" for ol in m.lines[3:]], [False, False])

    def test_struct_patterns_in_heads(self):
        src = (
            "fn main() {\n"
            "    for Pair { a, b } in pairs {\n"
            '        println!("{a}");\n'
            "    }\n"
            "    if let Trap::PageFault { addr } = t {\n"
            '        println!("{addr}");\n'
            "    }\n"
            "}\n"
        )
        s3, s6 = [s for s in program(src).sites]
        self.assertFalse(s3.once)
        self.assertTrue(s6.cond)

    def test_multiline_value_withdraws_wildcard_links(self):
        src = ('fn main() {\n    println!("== a ==");\n    println!("{}", two_lines);\n'
               '    println!("{}", one_line);\n}\n')
        m = pa.map_run(program(src), "== a ==\nx\ny\nz\n")
        self.assertEqual([ol.conf == "confident" for ol in m.lines], [True, False, False, False])

    def test_out_of_order_match_is_withdrawn(self):
        src = (
            "fn main() {\n"                                          # 1
            '    println!("== a ==");\n'                              # 2
            "    for (k, v) in [(\"a\", 1), (\"size\", 8)] {\n"   # 3
            '        println!("  {k} = {v}");\n'                     # 4
            "    }\n"                                                  # 5
            '    println!("  size = {}", 8);\n'                       # 6
            "}\n"
        )
        m = pa.map_run(program(src), "== a ==\n  a = 1\n  size = 8\n  size = 8\n")
        wrong = [(i, ol.src) for i, ol in enumerate(m.lines)
                 if ol.conf == "confident" and ol.src == 6 and i != 3]
        self.assertEqual(wrong, [])
        self.assertFalse(m.lines[3].conf == "confident" and m.lines[3].src == 4)

    def test_multiline_panic_message(self):
        src = ('fn main() {\n    println!("== p ==");\n    assert_eq!(1, 2);\n'
               '    println!("  {}", x);\n}\n')
        out = ("== p ==\nthread 'main' panicked at src/bin/t.rs:3:5:\n"
               "assertion `left == right` failed\n  left: 1\n right: 2\n"
               "note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\n")
        m = pa.map_run(program(src), out)
        self.assertEqual([(ol.src, ol.kind) for ol in m.lines[1:]], [(3, "panic")] * 5)

    def test_terminators_skip_branches_that_did_not_run(self):
        src = (
            "fn main() {\n"                               # 1
            '    println!("== a ==");\n'                  # 2
            '    print!("result: ");\n'                   # 3
            "    if ok {\n"                               # 4
            '        print!("fine");\n'                   # 5
            "    } else {\n"                              # 6
            '        print!("broken");\n'                 # 7
            "    }\n"                                     # 8
            "    println!();\n"                           # 9
            "}\n"
        )
        m = pa.map_run(program(src), "== a ==\nresult: fine\n")
        self.assertEqual((m.lines[1].src, m.lines[1].also), (3, [5, 9]))

    def test_wildcard_println_is_not_a_verified_terminator(self):
        src = ('fn main() {\n    println!("== a ==");\n    print!("value is ");\n    helper();\n'
               '    println!("{}", later);\n}\n'
               'fn helper() {\n    println!("42");\n}\n')
        m = pa.map_run(program(src), "== a ==\nvalue is 42\nlater\n")
        self.assertEqual(m.lines[1].also, [])

    def test_header_with_second_line(self):
        src = 'fn main() {\n    println!("== a ==\\n(subtitle)");\n    println!("(subtitle)");\n}\n'
        m = pa.map_run(program(src), "== a ==\n(subtitle)\n(subtitle)\n")
        self.assertEqual([ol.src for ol in m.lines], [2, 2, 3])

    def test_diag_rows_follow_the_file(self):
        rendered = ("error[E0277]: x\n  --> e0277.rs:4:5\n   |\n4  |     foo()?;\n   |\n"
                    "note: required by a bound in `Try`\n  --> /rustc/abc/library/core/src/ops/try_trait.rs:9:1\n"
                    "9  | pub trait Try {}\n")
        rows = pa.diag_rows(rendered, "e0277.rs", "e0277")
        self.assertIn('data-src="4"', rows)
        self.assertNotIn('data-src="9"', rows)

    def test_error_codes_in_output_are_not_cross_links(self):
        known = {"e0277": "e0277"}
        self.assertNotIn("<a", pa.linkify("error[E0277]: can't compare `[u8]` with `str`", known))
        self.assertIn('href="#e0277"', pa.linkify("RUN  ./show-errors.sh e0277", known))

    def test_publish_normalization(self):
        deck = ('        .back-link a:hover {\n        }\n\n        .reveal a.ex-link {\n            x: y;\n        }\n\n'
                '        .highlight-box {\n**RUN** `cargo run --bin a` <a class="ex-link" href="examples.html#a" '
                'target="cs326-examples">code + output &#8599;</a>\n')
        plain = '        .back-link a:hover {\n        }\n\n        .highlight-box {\n**RUN** `cargo run --bin a`\n'
        self.assertEqual(pa.normalize_deck(deck), plain)


# ---------------------------------------------------------------------------
# The local runner behind `pointat serve`
# ---------------------------------------------------------------------------

import http.client            # noqa: E402
import json as _json          # noqa: E402
import os                     # noqa: E402
import shutil                 # noqa: E402
import subprocess             # noqa: E402
import tempfile               # noqa: E402
import threading              # noqa: E402
import time                   # noqa: E402

sys.modules.setdefault("pointat", pa)
import pointat_serve as ps    # noqa: E402

HAVE_RUSTC = shutil.which("rustc") is not None


class SpawnCappedTest(unittest.TestCase):
    """The primitive that keeps a typed-in program from taking the machine."""

    def test_captures_merged_output(self):
        r = ps.spawn_capped(["/bin/sh", "-c", "echo out; echo err 1>&2"], ".", os.environ, 10)
        self.assertEqual(sorted(r["data"].decode().split()), ["err", "out"])
        self.assertEqual((r["exit"], r["timed_out"], r["truncated"]), (0, False, False))

    def test_output_is_capped_while_it_runs(self):
        r = ps.spawn_capped(["/bin/sh", "-c", "while :; do echo xxxxxxxxxxxxxxxx; done"],
                            ".", os.environ, 20, cap=8192)
        self.assertTrue(r["truncated"])
        self.assertLessEqual(len(r["data"]), 8192)

    def test_timeout_kills_the_process_group(self):
        tmp = Path(tempfile.mkdtemp())
        try:
            marker = tmp / "grandchild-was-still-running"
            r = ps.spawn_capped(["/bin/sh", "-c", f"(sleep 3; touch {marker}) & wait"],
                                tmp, os.environ, 0.4)
            self.assertTrue(r["timed_out"])
            time.sleep(3.2)
            self.assertFalse(marker.exists(), "a grandchild outlived its run")
        finally:
            shutil.rmtree(tmp, ignore_errors=True)


@unittest.skipUnless(HAVE_RUSTC, "needs rustc")
class CompileAndRunTest(unittest.TestCase):
    def build(self, source, args=()):
        tmp = Path(tempfile.mkdtemp())
        try:
            ps.stage(tmp, "demo", source)
            built = ps.compile_one(tmp, "demo", "2021", pa.run_env())
            if not built["compiled"]:
                return built, None
            return built, ps.run_once(tmp, "demo", list(args), pa.run_env(), 10)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    def test_runs_and_reports_argv_like_cargo_does(self):
        built, ran = self.build('fn main() { println!("{:?}", std::env::args().next()); }', ["a"])
        self.assertTrue(built["compiled"])
        self.assertIn("target/debug/demo", ran["output"])
        self.assertEqual(ran["exit"], 0)

    def test_diagnostics_have_the_captures_shape(self):
        built, ran = self.build("fn main() { let t: [u8; 1] = [0]; t[0] = 1; }")
        self.assertFalse(built["compiled"])
        self.assertIsNone(ran)
        d = built["diagnostics"][0]
        self.assertEqual(d["level"], "error")
        self.assertTrue(d["code"].startswith("E"))
        self.assertEqual(d["spans"][0]["ls"], 1)
        self.assertIn("demo.rs", built["rendered"])


class ServeTest(unittest.TestCase):
    """Nothing but a page this process served may make it compile anything."""

    @classmethod
    def setUpClass(cls):
        cls.httpd, cls.state = ps.build_server(["week04"], port=0, timeout=5)
        cls.thread = threading.Thread(target=cls.httpd.serve_forever, daemon=True)
        cls.thread.start()
        cls.port = cls.state.port

    @classmethod
    def tearDownClass(cls):
        cls.httpd.shutdown()
        cls.httpd.server_close()

    def ask(self, method, path, body=None, host=None, origin=None, token=None,
            ctype="application/json"):
        conn = http.client.HTTPConnection("127.0.0.1", self.port, timeout=60)
        headers = {"Host": host or f"127.0.0.1:{self.port}"}
        if origin is not None:
            headers["Origin"] = origin
        if token is not None:
            headers["X-Pointat-Token"] = token
        if body is not None:
            headers["Content-Type"] = ctype
        conn.request(method, path, body=body, headers=headers)
        r = conn.getresponse()
        data = r.read()
        conn.close()
        return r.status, r.headers, data

    def run_body(self, code="fn main() {}", stem="03_slot_search", **kw):
        return _json.dumps(dict({"week": "week04", "stem": stem, "code": code}, **kw))

    def test_health_and_page(self):
        code, _, data = self.ask("GET", "/health")
        self.assertEqual(code, 200)
        self.assertEqual(_json.loads(data)["weeks"], ["week04"])
        self.assertNotIn(self.state.token, data.decode())
        code, _, page = self.ask("GET", "/week04/examples.html")
        self.assertEqual(code, 200)
        self.assertIn(self.state.token, page.decode())
        self.assertIn('"runner": "local"', page.decode())

    def test_no_cors_headers_anywhere(self):
        for method, path in (("GET", "/health"), ("GET", "/week04/examples.html")):
            _, headers, _ = self.ask(method, path)
            self.assertEqual([k for k in headers.keys() if k.lower().startswith("access-control")], [])

    def test_refusals(self):
        body = self.run_body()
        tok = self.state.token
        cases = [
            ("no token", dict(token=None), 403),
            ("wrong token", dict(token="0" * 32), 403),
            ("bad host", dict(token=tok, host="evil.example:1"), 403),
            ("foreign origin", dict(token=tok, origin="https://evil.example"), 403),
            ("origin null", dict(token=tok, origin="null"), 403),
            ("text/plain", dict(token=tok, ctype="text/plain"), 415),
        ]
        for label, kw, want in cases:
            code, _, _ = self.ask("POST", "/run", body=body, **kw)
            self.assertEqual(code, want, f"{label} should be {want}")

    def test_get_and_options_cannot_run(self):
        self.assertEqual(self.ask("GET", "/run?code=fn+main(){}")[0], 404)
        self.assertEqual(self.ask("OPTIONS", "/run")[0], 405)
        self.assertEqual(self.ask("GET", "/../tools/pointat.py")[0], 404)

    def test_bad_requests(self):
        tok = self.state.token
        self.assertEqual(self.ask("POST", "/run", body="not json", token=tok)[0], 400)
        self.assertEqual(self.ask("POST", "/run", body=self.run_body(stem="../../etc/passwd"),
                                  token=tok)[0], 404)
        self.assertEqual(self.ask("POST", "/run", body=self.run_body(stem="99_nope"), token=tok)[0], 404)
        self.assertEqual(self.ask("POST", "/run", body=self.run_body(code=""), token=tok)[0], 400)
        self.assertEqual(self.ask("POST", "/run", body="x" * (ps.SOURCE_CAP + 1), token=tok)[0], 413)
        self.assertEqual(self.ask("POST", "/run", body=self.run_body(args=["ok"] * 99),
                                  token=tok)[0], 400)

    @unittest.skipUnless(HAVE_RUSTC, "needs rustc")
    def test_happy_path_compiles_and_runs(self):
        body = self.run_body(code='fn main() { println!("== a ==\nhi"); }')
        code, _, data = self.ask("POST", "/run", body=body, token=self.state.token,
                                 origin=f"http://127.0.0.1:{self.port}")
        self.assertEqual(code, 200)
        got = _json.loads(data)
        self.assertTrue(got["ok"] and got["compiled"])
        self.assertEqual(got["output"], "== a ==\nhi\n")
        self.assertEqual((got["exit"], got["runner"]), (0, "local"))
        self.assertEqual(got["file"], "src/bin/03_slot_search.rs")


class ServeCliTest(unittest.TestCase):
    def test_serve_is_a_subcommand_of_the_documented_spelling(self):
        with self.assertRaises(SystemExit) as caught:
            pa.main(["serve", "--help"])
        self.assertEqual(caught.exception.code, 0)


if __name__ == "__main__":
    unittest.main()
