/* pointat v2: which lines of a program are its `== section ==` headers.

   This is the only thing the page infers about edited code: the k-th
   `println!("== …")` at brace depth 1 of `main`, and the k-th `== ` line of
   the output that matches it.  Nothing here claims which line printed which
   line; that stays in Python, where the captured output is complete.

   tools/parity-bands.mjs loads this file in node and checks it against
   Python's own section table for every program in the corpus. */
(function (root) {
  "use strict";

// =======================================================================
// Scanning the edited source for its `== section ==` headers
// =======================================================================

function stringAt(src, i) {
  // "…", b"…", r"…", r#"…"#, br##"…"##  ->  {end, text} or null
  var m = /^(b?r(#*)"|b?")/.exec(src.slice(i, i + 12));
  if (!m) return null;
  var open = m[1], raw = open.indexOf("r") >= 0 && open.indexOf("r") < open.indexOf('"');
  var j = i + open.length;
  if (raw) {
    var close = '"' + m[2];
    var e = src.indexOf(close, j);
    if (e < 0) return { end: src.length, text: src.slice(j) };
    return { end: e + close.length, text: src.slice(j, e) };
  }
  var out = "";
  while (j < src.length) {
    var c = src[j];
    if (c === "\\") {
      var nx = src[j + 1];
      out += nx === "n" ? "\n" : nx === "t" ? "\t" : nx === "r" ? "\r" : nx === "0" ? "\0" : nx;
      j += 2;
      continue;
    }
    if (c === '"') return { end: j + 1, text: out };
    out += c;
    j++;
  }
  return { end: src.length, text: out };
}

function charEnd(src, i) {
  // A char literal or a lifetime at src[i] === "'" -> the offset past it, or 0
  if (src[i + 1] === "\\") {
    var e = src.indexOf("'", src.slice(i + 1, i + 3) === "\\'" ? i + 3 : i + 2);
    return e > i && e - i <= 12 ? e + 1 : 0;
  }
  if (src[i + 2] === "'" && src[i + 1] !== "\n") return i + 3;
  return 0;                        // a lifetime: let the scanner walk on
}

function bandText(lit) {
  // The literal part of a header, up to its first placeholder.
  var s = lit.replace(/^\n+/, ""), out = "", full = true;
  for (var i = 0; i < s.length; i++) {
    if (s[i] === "{" || s[i] === "}") {
      if (s[i + 1] === s[i]) { out += s[i]; i++; continue; }
      full = false;
      break;
    }
    out += s[i];
  }
  return { text: out.replace(/\s+$/, ""), full: full };
}

/* The one inference this file makes: which lines of `src` are the k-th
   `println!("== …")` of main, found by brace depth with strings, chars and
   comments skipped.  Checked against Python's own header list for every
   program in the corpus by tools/parity-bands.mjs. */
function scanSections(src) {
  var n = src.length, i = 0, line = 1, depth = 0, mainDepth = -1, seenMain = false;
  var headers = [], mainOpen = 0, mainClose = 0;
  function count(text) {
    for (var k = 0; k < text.length; k++) if (text[k] === "\n") line++;
  }
  while (i < n) {
    var c = src[i];
    if (c === "\n") { line++; i++; continue; }
    if (c === "/" && src[i + 1] === "/") {
      var e = src.indexOf("\n", i);
      i = e < 0 ? n : e;
      continue;
    }
    if (c === "/" && src[i + 1] === "*") {
      var d = 1;
      i += 2;
      while (i < n && d > 0) {
        if (src[i] === "\n") line++;
        else if (src[i] === "/" && src[i + 1] === "*") { d++; i++; }
        else if (src[i] === "*" && src[i + 1] === "/") { d--; i++; }
        i++;
      }
      continue;
    }
    if (c === '"' || c === "r" || c === "b") {
      var str = stringAt(src, i);
      if (str) { count(src.slice(i, str.end)); i = str.end; continue; }
    }
    if (c === "'") {
      var ce = charEnd(src, i);
      if (ce) { i = ce; continue; }
    }
    if (c === "{") {
      depth++;
      if (seenMain && mainDepth < 0) { mainDepth = depth; mainOpen = line; }
      i++;
      continue;
    }
    if (c === "}") {
      if (mainDepth > 0 && depth === mainDepth) { mainClose = line; mainDepth = -2; seenMain = false; }
      depth--;
      i++;
      continue;
    }
    if (/[A-Za-z_]/.test(c)) {
      var w = /^[A-Za-z_][0-9A-Za-z_]*/.exec(src.slice(i, i + 80))[0];
      if (mainDepth === -1 && w === "fn" && /^fn\s+main\s*\(/.test(src.slice(i, i + 40))) seenMain = true;
      if (w === "println" && src[i + 7] === "!" && mainDepth > 0 && depth === mainDepth) {
        // Look ahead for the literal without moving the line counter: the
        // scan position rewinds to just after `println`, so any newline in
        // between would otherwise be counted twice.
        var hline = line, j = i + 8;
        while (j < n && /[\s(]/.test(src[j])) j++;
        var lit = stringAt(src, j);
        if (lit && /^\n*== /.test(lit.text)) {
          var b = bandText(lit.text);
          headers.push({ line: hline, text: b.text, full: b.full });
        }
      }
      i += w.length;
      continue;
    }
    i++;
  }
  if (!mainClose) mainClose = line;
  return { headers: headers, mainOpen: mainOpen, mainClose: mainClose };
}

function codeBands(scan, lineCount) {
  // line number -> {k, first}
  var bands = {}, h = scan.headers;
  if (!h.length || !scan.mainOpen) return bands;
  var edges = [{ k: 0, from: scan.mainOpen + 1 }];
  for (var i = 0; i < h.length; i++) edges.push({ k: i + 1, from: h[i].line });
  for (var e = 0; e < edges.length; e++) {
    var to = e + 1 < edges.length ? edges[e + 1].from - 1 : Math.min(scan.mainClose - 1, lineCount);
    if (edges[e].k === 0 && to < edges[e].from) continue;
    for (var L = edges[e].from; L <= to; L++) bands[L] = { k: edges[e].k, first: L === edges[e].from };
  }
  return bands;
}

function outputBands(headers, lines) {
  // output line index -> section number, following Python's rule: the k-th
  // `== ` line that matches header k, in order.
  var at = {}, next = 0;
  for (var i = 0; i < lines.length; i++) {
    var t = lines[i].replace(/\s+$/, "");
    if (t.indexOf("== ") !== 0) continue;
    for (var k = next; k < headers.length; k++) {
      var h = headers[k];
      if (h.full ? t === h.text : t.indexOf(h.text) === 0) {
        at[i] = k + 1;
        next = k + 1;
        break;
      }
    }
  }
  return at;
}

  var API = {
    stringAt: stringAt, charEnd: charEnd, bandText: bandText,
    scanSections: scanSections, codeBands: codeBands, outputBands: outputBands
  };
  root.POINTAT_BANDS = API;
  if (typeof module !== "undefined" && module.exports) module.exports = API;
})(typeof window !== "undefined" ? window : globalThis);
