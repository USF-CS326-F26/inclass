/* pointat v2: edit an example in the page, run it, see what it prints.

   The captured grid is Python's and is never modified: entering edit mode
   detaches it and keeps the node, so Revert puts the original back exactly.
   A fresh run shows section bands only, never per-line links: the bands come
   from a scan of the edited source, which cannot claim more than it knows. */
(function () {
  "use strict";
  var CFG = window.POINTAT || {};
  var doc = document;
  var APP = window.__pointat || {};
  var slice = function (xs) { return Array.prototype.slice.call(xs); };
  var states = {};                  // stem -> state
  var pgVersion = null;             // the Playground's rustc, once asked

  // =========================================================================
  // The section scanner lives in tools/pointat.bands.js, where node can load
  // it too (see tools/parity-bands.mjs).
  // =========================================================================

  var BANDS = window.POINTAT_BANDS || {};
  var scanSections = BANDS.scanSections, codeBands = BANDS.codeBands, outputBands = BANDS.outputBands;

  // =========================================================================
  // Small DOM helpers
  // =========================================================================

  function el(tag, cls, text) {
    var e = doc.createElement(tag);
    if (cls) e.className = cls;
    if (text !== undefined) e.textContent = text;
    return e;
  }

  function style(k) {
    var p = CFG.palette || [];
    return p.length ? p[k === 0 ? 0 : 1 + ((k - 1) % (p.length - 1))] : "";
  }

  /* The source, read back out of the rows Python rendered: the line number is
     a CSS ::before, so the row's text is the line itself. */
  function capturedSource(art) {
    var rows = slice(art.querySelectorAll(".grid .cell.code [data-l]"));
    rows.sort(function (a, b) { return a.dataset.l - b.dataset.l; });
    return rows.map(function (r) {
      var c = r.cloneNode(true);
      slice(c.querySelectorAll(".badge, .more")).forEach(function (n) { n.remove(); });
      return c.textContent;
    }).join("\n") + "\n";
  }

  /* The committed output for one run, read back out of the rendered rows:
     [{sec, text}], in order, exactly as the page shows it. */
  function capturedOutput(grid, run) {
    var out = [];
    if (!grid) return out;
    slice(grid.querySelectorAll(".cell.out")).forEach(function (cell) {
      var sec = cell.dataset.sec === undefined ? null : parseInt(cell.dataset.sec, 10);
      var block = cell.querySelector('.run[data-run="' + run + '"]') || cell;
      slice(block.querySelectorAll(".ol")).forEach(function (row) {
        var c = row.cloneNode(true);
        slice(c.querySelectorAll(".badge, .by")).forEach(function (n) { n.remove(); });
        out.push({ sec: sec, text: c.textContent });
      });
    });
    return out;
  }

  /* A line diff, longest common subsequence, as pairs:
     {left, right} with null where a side has nothing. */
  function diffLines(a, b) {
    var n = a.length, m = b.length;
    if (n * m > 600000) {                  // far past anything worth aligning
      return a.map(function (l) { return { left: l, right: null }; })
        .concat(b.map(function (r) { return { left: null, right: r }; }));
    }
    var lcs = [];
    for (var i = 0; i <= n; i++) lcs.push(new Uint32Array(m + 1));
    for (i = n - 1; i >= 0; i--) {
      for (var j = m - 1; j >= 0; j--) {
        lcs[i][j] = a[i] === b[j] ? lcs[i + 1][j + 1] + 1
          : Math.max(lcs[i + 1][j], lcs[i][j + 1]);
      }
    }
    var pairs = [];
    i = 0;
    j = 0;
    while (i < n && j < m) {
      if (a[i] === b[j]) { pairs.push({ left: a[i], right: b[j], same: true }); i++; j++; }
      else if (lcs[i + 1][j] >= lcs[i][j + 1]) { pairs.push({ left: a[i], right: null }); i++; }
      else { pairs.push({ left: null, right: b[j] }); j++; }
    }
    while (i < n) pairs.push({ left: a[i++], right: null });
    while (j < m) pairs.push({ left: null, right: b[j++] });
    return pairs;
  }

  function bySection(rows) {
    // Blank lines are dropped: the captured view trims the blank a "\n== …"
    // header prints, so keeping them would mark every section as changed.
    var map = {}, order = [];
    rows.forEach(function (r) {
      if (!r.text.trim()) return;
      var k = r.sec === null || r.sec === undefined ? 0 : r.sec;
      if (!map[k]) { map[k] = []; order.push(k); }
      map[k].push(r.text);
    });
    return { map: map, order: order };
  }

  function renderCompare(st) {
    var box = st.cmp;
    box.textContent = "";
    if (!st.last || !st.last.compiled) {
      box.appendChild(el("div", "fmsg", st.last
        ? "your edit does not compile, so there is nothing to compare yet"
        : "run it first (\u2318\u23ce), then compare"));
      return;
    }
    // st.grid is the captured grid, detached while the editor is up.
    var committed = bySection(capturedOutput(st.grid, st.run));
    var text = st.last.merged !== undefined ? st.last.merged : st.last.stdout;
    var lines = (text || "").split("\n");
    if (lines.length && lines[lines.length - 1] === "") lines.pop();
    var at = outputBands(st.scan.headers, lines);
    var freshRows = [], sec = 0;
    lines.forEach(function (line, i) {
      if (at[i] !== undefined) sec = at[i];
      freshRows.push({ sec: sec, text: line });
    });
    var fresh = bySection(freshRows);

    var head = el("div", "cmphead");
    head.appendChild(el("span", "side", "committed \u00b7 " + (CFG.captured_rustc || "the capture")));
    head.appendChild(el("span", "side", "yours \u00b7 " + (st.last.runner || "")));
    box.appendChild(head);

    var seen = {}, sections = [];
    committed.order.concat(fresh.order).forEach(function (k) {
      if (!seen[k]) { seen[k] = true; sections.push(k); }
    });
    sections.sort(function (x, y) { return x - y; });
    var changed = 0;
    var grid = el("div", "cmpgrid");
    sections.forEach(function (k) {
      var pairs = diffLines(committed.map[k] || [], fresh.map[k] || []);
      var left = el("div", "cmpcell" + (k ? " sec" : "")), right = el("div", "cmpcell" + (k ? " sec" : ""));
      if (k) {
        left.setAttribute("style", style(k));
        right.setAttribute("style", style(k));
      }
      pairs.forEach(function (pr, idx) {
        if (!pr.same) changed++;
        var l = el("div", "cmprow" + (pr.left === null ? " none" : pr.same ? "" : " gone"));
        var r = el("div", "cmprow" + (pr.right === null ? " none" : pr.same ? "" : " new"));
        if (k && idx === 0) {
          l.appendChild(el("span", "badge", String(k)));
          r.appendChild(el("span", "badge", String(k)));
        }
        l.appendChild(doc.createTextNode(pr.left === null ? "" : pr.left || "\u00a0"));
        r.appendChild(doc.createTextNode(pr.right === null ? "" : pr.right || "\u00a0"));
        left.appendChild(l);
        right.appendChild(r);
      });
      grid.appendChild(left);
      grid.appendChild(right);
    });
    box.appendChild(grid);
    box.appendChild(el("div", "fmsg", changed
      ? changed + (changed === 1 ? " line differs" : " lines differ") + " from the committed run"
      : "every line is the same as the committed run"));
    if ((st.art.dataset.notes || "").indexOf("nondet") >= 0) {
      box.appendChild(el("div", "fmsg", "this program prints addresses or capacities, " +
        "so those lines differ on every run, edit or no edit."));
    }
    if (CFG.captured_rustc && st.last.rustc && st.last.rustc !== CFG.captured_rustc) {
      box.appendChild(el("div", "fmsg", "different compilers: the capture came from " +
        CFG.captured_rustc + ", your run from " + st.last.rustc + "."));
    }
  }

  // =========================================================================
  // The editor
  // =========================================================================

  function insert(ta, text) {
    // Keep the browser's own undo history: never assign .value.
    ta.focus();
    if (!doc.execCommand("insertText", false, text)) {
      var s = ta.selectionStart, e = ta.selectionEnd;
      ta.value = ta.value.slice(0, s) + text + ta.value.slice(e);
      ta.selectionStart = ta.selectionEnd = s + text.length;
    }
  }

  function replaceAll(ta, text) {
    ta.focus();
    ta.setSelectionRange(0, ta.value.length);
    insert(ta, text);
  }

  function onEditorKey(st, e) {
    var ta = st.ta;
    if (e.isComposing || e.keyCode === 229) return;
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") { e.preventDefault(); doRun(st); return; }
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    if (e.key === "Escape") { e.preventDefault(); ta.blur(); return; }
    var v = ta.value, s = ta.selectionStart, t = ta.selectionEnd;
    var ls = v.lastIndexOf("\n", s - 1) + 1;
    if (e.key === "Tab") {
      e.preventDefault();
      if (!e.shiftKey && s === t) { insert(ta, "    "); return; }
      var end = v.indexOf("\n", t);
      end = end < 0 ? v.length : end;
      var block = v.slice(ls, end);
      var lines = block.split("\n").map(function (l) {
        return e.shiftKey ? l.replace(/^ {1,4}/, "") : (l.length ? "    " + l : l);
      });
      ta.setSelectionRange(ls, end);
      insert(ta, lines.join("\n"));
      ta.setSelectionRange(ls, ls + lines.join("\n").length);
      return;
    }
    if (e.key === "Enter" && !e.shiftKey && s === t) {
      e.preventDefault();
      var indent = /^[ \t]*/.exec(v.slice(ls))[0];
      var deeper = /[{(\[]\s*$/.test(v.slice(ls, s)) ? "    " : "";
      insert(ta, "\n" + indent + deeper);
      return;
    }
  }

  function paintGutter(st) {
    var src = st.ta.value;
    var lines = src.split("\n");
    if (lines.length && lines[lines.length - 1] === "") lines.pop();
    var scan = scanSections(src);
    st.scan = scan;
    var bands = codeBands(scan, lines.length);
    var gut = st.gut;
    gut.textContent = "";
    for (var L = 1; L <= lines.length; L++) {
      var row = el("div", "gl");
      row.dataset.l = String(L);
      var b = bands[L];
      if (b) {
        row.className = "gl sec";
        row.setAttribute("style", style(b.k));
        if (b.first) row.appendChild(el("span", "badge", String(b.k)));
      }
      gut.appendChild(row);
    }
    st.status.textContent = scan.headers.length
      ? scan.headers.length + (scan.headers.length === 1 ? " section" : " sections")
      : "no == sections ==";
  }

  // =========================================================================
  // The two runners
  // =========================================================================

  function withTimeout(promise, ms, ctl) {
    return new Promise(function (resolve, reject) {
      var t = setTimeout(function () {
        if (ctl) ctl.abort();
        reject(new Error("timeout"));
      }, ms);
      promise.then(function (v) { clearTimeout(t); resolve(v); },
                   function (e) { clearTimeout(t); reject(e); });
    });
  }

  function post(url, body, headers, ms) {
    var ctl = typeof AbortController !== "undefined" ? new AbortController() : null;
    var opts = { method: "POST", mode: "cors", headers: headers, body: JSON.stringify(body) };
    if (ctl) opts.signal = ctl.signal;
    return withTimeout(fetch(url, opts).then(function (r) {
      return r.text().then(function (text) {
        var json = null;
        try { json = JSON.parse(text); } catch (err) { /* not json */ }
        return { status: r.status, body: json, text: text };
      });
    }), ms, ctl);
  }

  function playgroundVersion() {
    if (pgVersion !== null) return Promise.resolve(pgVersion);
    return withTimeout(fetch("https://play.rust-lang.org/meta/versions", { mode: "cors" })
      .then(function (r) { return r.json(); })
      .then(function (j) {
        var v = j && j.stable && j.stable.rustc;          // {version, hash, date}
        pgVersion = v ? "rustc " + (v.version || v) + (v.date ? " (" + v.date + ")" : "") : "";
        return pgVersion;
      }), 4000, null).catch(function () { pgVersion = ""; return ""; });
  }

  var CARGO_NOISE = /^\s*(Compiling|Finished|Running|Updating|Locking|Blocking|Downloaded|Adding|warning: unused manifest key)\b/;

  function runLocal(st, source) {
    return post(CFG.endpoint, {
      week: CFG.week, stem: st.stem, run: st.run, code: source, args: st.args
    }, { "Content-Type": "application/json", "X-Pointat-Token": CFG.token },
    (CFG.timeout || 10) * 1000 + 25000).then(function (r) {
      var b = r.body;
      if (!b || b.ok !== true) {
        return { error: b && b.error === "busy" ? "the local runner is busy with another run"
                                                : "the local runner said " + ((b && b.error) || r.status) };
      }
      return {
        runner: "local toolchain", rustc: b.rustc, file: b.file, cmd: b.cmd, args: b.args,
        compiled: b.compiled, diagText: b.rendered, merged: b.output,
        exit: b.exit, signal: b.signal, timedOut: b.timed_out, truncated: b.truncated
      };
    });
  }

  function runPlayground(st, source) {
    return playgroundVersion().then(function (ver) {
      return post(CFG.endpoint, {
        channel: CFG.channel || "stable", mode: CFG.mode || "debug",
        edition: CFG.edition || "2021", crateType: "bin", tests: false,
        backtrace: false, code: source
      }, { "Content-Type": "application/json" }, (CFG.timeout || 15) * 1000).then(function (r) {
        var b = r.body;
        if (r.status !== 200 || !b) {
          return { error: "the Rust Playground said " + r.status + ((b && b.error) ? ": " + b.error : "") };
        }
        var lines = (b.stderr || "").split("\n").filter(function (l) { return !CARGO_NOISE.test(l); });
        var diag = lines.join("\n").replace(/^\n+/, "").replace(/\n+$/, "");
        var m = /Exited with status (\d+)/.exec(b.exitDetail || "");
        return {
          runner: "Rust Playground", rustc: ver, file: "src/main.rs",
          cmd: "cargo run", args: [],
          compiled: !/^error(\[|:)/m.test(diag),
          diagText: diag, stdout: b.stdout || "",
          exit: m ? parseInt(m[1], 10) : (b.success ? 0 : null),
          detail: b.exitDetail || "", timedOut: false, truncated: false
        };
      });
    });
  }

  // =========================================================================
  // Rendering a fresh run
  // =========================================================================

  function diagRow(line, file, st) {
    var cls = "fdl", m;
    if (/^(error|warning)(\[E\d{4}\])?: /.test(line)) cls += /^error/.test(line) ? " err hd" : " warn hd";
    else if (/^(note|help): /.test(line)) cls += " note hd";
    else if ((m = /^(\s*)--> (\S+?):(\d+):(\d+)/.exec(line))) cls += " loc";
    else if (/^\s*\d+\s*\|/.test(line)) cls += " src";
    else if (/^\s*\|/.test(line)) cls += " gut";
    else if (/^\s*= /.test(line)) cls += " eq";
    var row = el("div", cls);
    if (m) {
      row.appendChild(doc.createTextNode(m[1] + "--> "));
      var a = el("a", "jump", m[2] + ":" + m[3] + ":" + m[4]);
      a.href = "#";
      a.dataset.l = m[3];
      row.appendChild(a);
      row.appendChild(doc.createTextNode(line.slice(m[0].length)));
    } else {
      row.textContent = line;
    }
    return row;
  }

  function freshNotes(st, res) {
    var notes = [], flags = (st.art.dataset.notes || "").split(" ");
    var local = res.runner === "local toolchain";
    if (!local && flags.indexOf("argv") >= 0 && (st.args || []).length) {
      notes.push("The Playground runs `cargo run` with no arguments, so this run did not get " +
                 st.args.join(" ") + ". Run it under `pointat serve` to pass them.");
    }
    if (!local && flags.indexOf("typename") >= 0) {
      notes.push("On the Playground the crate is called `playground`, so type names differ from the capture.");
    }
    if (flags.indexOf("nondet") >= 0) {
      notes.push("Addresses and capacities differ every run; the captured numbers were a different run.");
    }
    if (CFG.captured_rustc && res.rustc && res.rustc !== CFG.captured_rustc) {
      notes.push("The captured output came from " + CFG.captured_rustc + ".");
    }
    return notes;
  }

  function renderFresh(st, res) {
    var box = st.fresh;
    box.textContent = "";
    var when = new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    var head = el("div", "fhead");
    head.appendChild(el("span", "who", res.runner + (res.rustc ? " · " + res.rustc : "")));
    head.appendChild(el("span", "when", "\u00b7 ran " + when));
    box.appendChild(head);

    if (res.error) {
      box.appendChild(el("div", "fmsg bad", res.error));
      box.appendChild(el("div", "fmsg", CFG.runner === "local"
        ? "Is the helper still running in the terminal?"
        : "Pressing Run sends the code to play.rust-lang.org, which needs a network connection."));
      st.freshBands = null;
      return;
    }

    if (res.diagText) {
      var d = el("div", "fdiag" + (res.compiled ? " warn" : " err"));
      d.appendChild(el("div", "flabel", res.compiled ? "cargo build says" : "it does not compile"));
      var body = el("div", "fdiagbody");
      res.diagText.split("\n").forEach(function (l) { body.appendChild(diagRow(l, res.file, st)); });
      d.appendChild(body);
      box.appendChild(d);
    }

    if (!res.compiled) {
      st.freshBands = null;
      paintGutter(st);
      return;
    }

    var text = res.merged !== undefined ? res.merged : res.stdout;
    var lines = (text || "").split("\n");
    if (lines.length && lines[lines.length - 1] === "") lines.pop();
    var MAX_ROWS = 2000, dropped = 0;
    if (lines.length > MAX_ROWS) {
      dropped = lines.length - MAX_ROWS;
      lines = lines.slice(0, MAX_ROWS);
    }
    var at = outputBands(st.scan.headers, lines);
    var sec = 0, cell = null;
    var out = el("div", "fout");
    lines.forEach(function (line, i) {
      if (at[i] !== undefined) { sec = at[i]; cell = null; }
      if (!cell) {
        cell = el("div", sec ? "fsec" : "fsec plain");
        if (sec) cell.setAttribute("style", style(sec));
        cell.dataset.sec = String(sec);
        out.appendChild(cell);
      }
      var row = el("div", "frow" + (at[i] !== undefined ? " hd" : ""));
      if (at[i] !== undefined && sec) row.appendChild(el("span", "badge", String(sec)));
      row.appendChild(doc.createTextNode(line || " "));
      cell.appendChild(row);
    });
    if (!lines.length) out.appendChild(el("div", "fmsg", "it printed nothing"));
    box.appendChild(out);
    if (dropped) {
      box.appendChild(el("div", "fmsg", "the first " + MAX_ROWS + " lines are shown; " +
        dropped + " more are not"));
    }

    if (res.runner === "Rust Playground" && res.diagText && res.compiled) {
      box.appendChild(el("div", "fmsg", "stderr is shown above: the Playground returns the two streams " +
        "separately, so they cannot be interleaved the way the capture is."));
    }
    var bits = [];
    if (res.truncated) bits.push("stopped after 256 KiB of output");
    else if (res.timedOut) bits.push("stopped after " + (CFG.timeout || 10) + " seconds");
    else if (res.signal) bits.push("killed by signal " + res.signal);
    else bits.push("exit status " + (res.exit === null || res.exit === undefined ? "?" : res.exit));
    box.appendChild(el("div", "fexit", bits.join(" · ")));
    freshNotes(st, res).forEach(function (n) { box.appendChild(el("div", "fmsg", n)); });
    paintGutter(st);
  }

  // =========================================================================
  // Lifecycle: edit -> run -> compare -> revert
  // =========================================================================

  function stateFor(art) {
    if (states[art.id]) return states[art.id];
    var st = { art: art, stem: art.id, run: 1, args: [], showing: "captured" };
    try {
      var runs = JSON.parse(art.dataset.runs || "[]");
      st.runs = runs;
      st.args = (runs[0] && runs[0].args) || [];
    } catch (e) { st.runs = []; }
    states[art.id] = st;
    return st;
  }

  function build(st) {
    var art = st.art;
    st.source = capturedSource(art);
    st.grid = art.querySelector(".grid");

    var live = el("div", "live");
    var bar = el("div", "livebar");
    st.runBtn = el("button", "runbtn", "Run");
    st.runBtn.type = "button";
    st.runBtn.title = "run this code (⌘⏎ / Ctrl+Enter)";
    st.cmpBtn = el("button", "cmp", "Compare");
    st.cmpBtn.type = "button";
    st.cmpBtn.title = "the committed output beside yours (c)";
    st.revBtn = el("button", "rev", "Revert");
    st.revBtn.type = "button";
    st.revBtn.title = "back to the committed example";
    st.status = el("span", "estatus", "");
    var who = el("span", "ernr", CFG.runner === "local"
      ? "runs here · " + (CFG.rustc || "local toolchain")
      : "runs on the Rust Playground");
    st.mode = el("span", "emode", "editing");
    bar.appendChild(st.mode);
    bar.appendChild(st.runBtn);
    bar.appendChild(st.cmpBtn);
    bar.appendChild(st.revBtn);
    bar.appendChild(st.status);
    bar.appendChild(who);
    live.appendChild(bar);

    var panes = el("div", "epanes");
    var ed = el("div", "ed");
    st.gut = el("div", "egut");
    st.ta = doc.createElement("textarea");
    st.ta.className = "ta";
    st.ta.setAttribute("wrap", "off");
    st.ta.spellcheck = false;
    st.ta.setAttribute("autocapitalize", "off");
    st.ta.setAttribute("autocomplete", "off");
    st.ta.setAttribute("autocorrect", "off");
    st.ta.value = st.source;
    ed.appendChild(st.gut);
    ed.appendChild(st.ta);
    st.fresh = el("div", "fresh");
    st.fresh.appendChild(el("div", "fmsg", CFG.runner === "local"
      ? "Edit the code, then press Run (⌘⏎). It compiles and runs here."
      : "Edit the code, then press Run (⌘⏎). It compiles and runs on the Rust Playground."));
    panes.appendChild(ed);
    panes.appendChild(st.fresh);
    live.appendChild(panes);
    st.cmp = el("div", "cmpview");
    live.appendChild(st.cmp);
    live.dataset.view = "edit";
    st.live = live;

    st.ta.addEventListener("keydown", function (e) { onEditorKey(st, e); });
    st.ta.addEventListener("input", function () { paintGutter(st); });
    st.ta.addEventListener("scroll", function () { st.gut.scrollTop = st.ta.scrollTop; });
    st.runBtn.addEventListener("click", function () { doRun(st); });
    st.cmpBtn.addEventListener("click", function () { toggleCompare(st); });
    st.revBtn.addEventListener("click", function () { revert(st); });
    st.fresh.addEventListener("click", function (e) {
      var a = e.target.closest ? e.target.closest("a.jump") : null;
      if (!a) return;
      e.preventDefault();
      jumpTo(st, parseInt(a.dataset.l, 10));
    });
    return st;
  }

  function jumpTo(st, line) {
    if (!line) return;
    var v = st.ta.value.split("\n"), at = 0;
    for (var i = 0; i < line - 1 && i < v.length; i++) at += v[i].length + 1;
    st.ta.focus();
    st.ta.setSelectionRange(at, at + (v[line - 1] || "").length);
    var row = st.gut.querySelector('.gl[data-l="' + line + '"]');
    if (row) st.ta.scrollTop = Math.max(0, row.offsetTop - st.ta.clientHeight / 2);
  }

  function showLive(st) {
    if (st.showing === "live") return;
    if (APP.unpin) APP.unpin();
    if (APP.unfocus) APP.unfocus();
    if (st.grid.parentNode) st.grid.parentNode.replaceChild(st.live, st.grid);
    st.showing = "live";
    st.art.dataset.mode = "editing";
    if (APP.rescan) APP.rescan();
    if (st.strip) { st.strip.remove(); st.strip = null; }
  }

  function showCaptured(st) {
    if (st.showing !== "live") return;
    if (st.live.parentNode) st.live.parentNode.replaceChild(st.grid, st.live);
    st.showing = "captured";
    st.art.dataset.mode = "compare";
    if (APP.rescan) APP.rescan();
  }

  function enterEdit(art) {
    var st = states[art.id] && states[art.id].live ? states[art.id] : build(stateFor(art));
    showLive(st);
    paintGutter(st);
    var first = st.scan.headers[0];
    st.ta.focus();
    jumpTo(st, first ? first.line : 1);
    return st;
  }

  function toggleCompare(st) {
    showLive(st);
    var to = st.live.dataset.view === "compare" ? "edit" : "compare";
    st.live.dataset.view = to;
    st.mode.textContent = to === "compare" ? "comparing" : "editing";
    st.cmpBtn.setAttribute("aria-pressed", to === "compare" ? "true" : "false");
    if (to === "compare") renderCompare(st);
  }

  function revert(st) {
    showCaptured(st);
    if (st.live) st.live.remove();
    delete st.art.dataset.mode;
    if (st.strip) { st.strip.remove(); st.strip = null; }
    delete states[st.art.id];
    if (APP.rescan) APP.rescan();
  }

  function doRun(st) {
    var source = st.ta.value;
    st.runBtn.disabled = true;
    st.fresh.textContent = "";
    var head = el("div", "fhead");
    head.appendChild(el("span", "who", CFG.runner === "local"
      ? "compiling here…" : "compiling on the Rust Playground…"));
    st.fresh.appendChild(head);
    st.fresh.appendChild(el("div", "fmsg", "this takes a second or two"));
    var call = CFG.runner === "local" ? runLocal(st, source) : runPlayground(st, source);
    call.then(function (res) {
      st.last = res;
      renderFresh(st, res);
      if (st.live.dataset.view === "compare") renderCompare(st);
    }, function (err) {
      renderFresh(st, { error: err && err.message === "timeout"
        ? "no answer within " + (CFG.timeout || 15) + " seconds"
        : "could not reach the runner (" + ((err && err.message) || "network error") + ")" });
    }).then(function () { st.runBtn.disabled = false; });
  }

  // A live edit must not masquerade as the captured page when the deck's RUN
  // link brings us back to this program later.
  function strip(st) {
    if (st.strip) return;
    var s = el("div", "editstrip");
    s.appendChild(el("span", "", "You have an unsaved edit of this program."));
    var show = el("button", "link", "show it");
    show.type = "button";
    var drop = el("button", "link", "discard");
    drop.type = "button";
    show.addEventListener("click", function () { showLive(st); });
    drop.addEventListener("click", function () { revert(st); });
    s.appendChild(show);
    s.appendChild(drop);
    st.art.querySelector("header.exh").appendChild(s);
    st.strip = s;
  }

  function onRoute() {
    var art = APP.current ? APP.current() : null;
    Object.keys(states).forEach(function (id) {
      var st = states[id];
      if (!st.live) return;
      if (art && art.id === id) {
        if (st.showing === "live") return;
        strip(st);
      } else if (st.showing === "live") {
        showCaptured(st);
        strip(st);
      }
    });
  }

  // =========================================================================
  // Wiring
  // =========================================================================

  doc.addEventListener("keydown", function (e) {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    var t = e.target;
    if (t && /^(SELECT|INPUT|TEXTAREA)$/.test(t.tagName)) return;
    var art = APP.current ? APP.current() : null;
    if (!art || art.dataset.kind !== "program") return;
    if (e.key === "i") {
      e.preventDefault();
      enterEdit(art);
    } else if (e.key === "c") {
      var st = states[art.id];
      if (st && st.live) {
        e.preventDefault();
        toggleCompare(st);
      }
    }
  });

  window.addEventListener("hashchange", function () { setTimeout(onRoute, 0); });

  // An Edit button in each program's header, made here so a page without
  // JavaScript never shows a control that cannot work.
  slice(doc.querySelectorAll('article.ex[data-kind="program"]')).forEach(function (art) {
    var b = el("button", "editbtn", "Edit and run");
    b.type = "button";
    b.title = "edit this program and run it (i)";
    b.addEventListener("click", function () { enterEdit(art); });
    var cmds = art.querySelector(".cmds");
    if (cmds) cmds.appendChild(b);
  });
})();
