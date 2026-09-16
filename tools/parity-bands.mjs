/* Check the page's section scanner against Python's.

   tools/pointat.bands.js is the only part of the page that infers anything
   about edited code: where a program's `== section ==` headers are, and which
   output lines belong to which section.  Python works the same thing out from
   the real lexer in tools/pointat.py.  They must agree on every program in the
   corpus, or the bands an instructor points at in edit mode mean nothing.

       node tools/parity-bands.mjs                 # every week
       node tools/parity-bands.mjs week04

   It shells out to `python3 tools/pointat.py --all --no-run --dump`, so it
   needs the committed captures but no network and no Rust toolchain.  Exit 0
   when every program and every run agrees. */
import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repo = resolve(here, "..");

const mod = { exports: {} };
new Function("module", "window", readFileSync(resolve(here, "pointat.bands.js"), "utf8"))(mod, {});
const { scanSections, outputBands } = mod.exports;

const weeks = process.argv.slice(2);
const args = ["tools/pointat.py", ...(weeks.length ? weeks : ["--all"]), "--no-run", "--dump"];
const raw = execFileSync("python3", args,
  { cwd: repo, maxBuffer: 64 * 1024 * 1024, stdio: ["ignore", "pipe", "ignore"] }).toString();
const dumps = [].concat(JSON.parse(raw));

let programs = 0, runs = 0, bad = 0;

for (const dump of dumps) {
  for (const prog of dump.programs) {
    programs++;
    const path = `${dump.week}/examples/src/bin/${prog.stem}.rs`;
    const source = readFileSync(resolve(repo, path), "utf8");
    const scan = scanSections(source);

    // Python's section table: {"0": [lo, hi], "1": [lo, hi], ...}
    const want = Object.keys(prog.sections)
      .map(Number).filter((k) => k > 0).sort((a, b) => a - b)
      .map((k) => prog.sections[String(k)][0]);
    const got = scan.headers.map((h) => h.line);
    if (want.join(",") !== got.join(",")) {
      bad++;
      console.log(`  ! ${path}: header lines differ\n      python ${want.join(",")}\n      page   ${got.join(",")}`);
    }

    for (const run of prog.runs) {
      runs++;
      const lines = run.lines.map((l) => l[2]);
      const at = outputBands(scan.headers, lines);
      let sec = 0;
      const mine = lines.map((_, i) => (at[i] !== undefined ? (sec = at[i]) : sec));
      const theirs = run.lines.map((l) => l[1]);
      const first = mine.findIndex((s, i) => s !== theirs[i]);
      if (first >= 0) {
        bad++;
        console.log(`  ! ${path} (${run.cmd}): output line ${first + 1} is section ` +
          `${mine[first]} here and ${theirs[first]} in python: ${JSON.stringify(lines[first].slice(0, 60))}`);
      }
    }
  }
}

console.log(`${programs} programs, ${runs} runs, ${bad} disagreement${bad === 1 ? "" : "s"}`);
process.exit(bad ? 1 : 0);
