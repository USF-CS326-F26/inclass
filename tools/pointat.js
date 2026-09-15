/* pointat: routing, section stepping, and click-to-pin.  Inlined into every
   weekNN/examples.html.  No history.pushState (it throws from file://). */
(function () {
  "use strict";
  var doc = document, root = doc.documentElement;
  var slice = function (xs) { return Array.prototype.slice.call(xs); };
  var arts = slice(doc.querySelectorAll("article.ex"));
  var byId = {};
  arts.forEach(function (a) { byId[a.id] = a; if (a.dataset.stem) byId[a.dataset.stem] = a; });
  var index = doc.getElementById("index");
  var pick = doc.getElementById("pick");
  var bar = doc.querySelector(".bar");
  var cur = null, secs = [], at = -1, pinned = [], saved = {};

  function get(k) { try { return localStorage.getItem("pointat." + k); } catch (e) { return null; } }
  function put(k, v) { try { localStorage.setItem("pointat." + k, String(v)); } catch (e) { /* private mode */ } }

  // ---- text size: the CSS breakpoints decide until someone presses + or - ----
  function currentFs() { return parseInt(getComputedStyle(root).getPropertyValue("--fs"), 10) || 17; }
  function setFs(v, save) {
    v = Math.max(10, Math.min(40, v));
    root.style.setProperty("--fs", v + "px");
    if (save) put("fs", v);
  }
  var storedFs = parseInt(get("fs"), 10);
  if (storedFs >= 10 && storedFs <= 40) setFs(storedFs, false);

  // ---- routing: #stem[/r2][/s3][/L17] ---------------------------------------
  function parseHash() {
    var h = location.hash.replace(/^#/, "");
    try { h = decodeURIComponent(h); } catch (e) { /* keep raw */ }
    var parts = h.split("/"), r = { id: parts[0] };
    for (var i = 1; i < parts.length; i++) {
      var m = /^([rsL])(\d+)$/.exec(parts[i]);
      if (m) r[m[1]] = parseInt(m[2], 10);
    }
    return r;
  }

  function route() {
    var r = parseHash(), a = byId[r.id] || null;
    var deep = r.s !== undefined || r.L !== undefined;
    if (a !== cur || (!a && !index.classList.contains("cur"))) {
      if (cur) saved[cur.id] = { y: window.scrollY, at: at };
      unpin(); unfocus();
      if (cur) cur.classList.remove("cur");
      cur = a;
      index.classList.toggle("cur", !a);
      if (a) {
        a.classList.add("cur");
        secs = slice(a.querySelectorAll(".cell.code.sec")).map(function (c) { return c.dataset.sec; });
        pick.value = a.id;
        var back = saved[a.id];
        if (back && !deep) {
          if (back.at >= 0) markSec(back.at);
          window.scrollTo(0, back.y);
        } else {
          window.scrollTo(0, 0);
        }
      } else {
        secs = [];
        pick.value = "index";
        var target = r.id && doc.getElementById(r.id);
        if (target && target !== index) target.scrollIntoView(); else window.scrollTo(0, 0);
      }
    }
    if (!a) return;
    if (r.r) setRun(r.r);
    if (r.s !== undefined) focusSec(secs.indexOf(String(r.s)));
    if (r.L) {
      var row = a.querySelector('.cell.code .ln[data-l="' + r.L + '"]');
      if (row) {
        var fold = row.closest("details");
        if (fold && !fold.open) fold.open = true;
        unpin();
        pinCode(row);
        row.scrollIntoView({ block: "center" });
      }
    }
    if (doc.activeElement === pick || !a.contains(doc.activeElement)) a.focus({ preventScroll: true });
  }

  function go(d) {
    var ids = arts.map(function (x) { return x.id; });
    var i = cur ? ids.indexOf(cur.id) + d : (d > 0 ? 0 : ids.length - 1);
    if (i < 0 || i >= ids.length) return;
    location.hash = ids[i];
  }

  // ---- runs ------------------------------------------------------------------
  function setRun(n) {
    if (!cur || !cur.querySelector('.run[data-run="' + n + '"]')) return;
    unpin();
    cur.dataset.run = String(n);
    slice(cur.querySelectorAll(".cmds .tab")).forEach(function (t) {
      t.setAttribute("aria-selected", t.dataset.run === String(n) ? "true" : "false");
    });
  }
  function cycleRun() {
    if (!cur) return;
    var tabs = cur.querySelectorAll(".cmds .tab");
    if (tabs.length) setRun(parseInt(cur.dataset.run || "1", 10) % tabs.length + 1);
  }

  // ---- section focus -----------------------------------------------------------
  function unfocus() {
    at = -1;
    if (!cur) return;
    cur.classList.remove("focus");
    slice(cur.querySelectorAll(".cell.on")).forEach(function (c) { c.classList.remove("on"); });
  }
  function markSec(i) {
    at = Math.max(0, Math.min(secs.length - 1, i));
    cur.classList.add("focus");
    slice(cur.querySelectorAll(".cell.on")).forEach(function (c) { c.classList.remove("on"); });
    var cells = cur.querySelectorAll('.cell.sec[data-sec="' + secs[at] + '"]');
    slice(cells).forEach(function (c) { c.classList.add("on"); });
    return cells;
  }
  function focusSec(i) {
    if (!cur || !secs.length || i < 0) return;
    var cells = markSec(i);
    if (cells[0]) cells[0].scrollIntoView({ block: "start", behavior: "smooth" });
  }
  function firstVisible() {
    // The first section that owns a real share of the screen, not a sliver under the bar.
    var top = bar.getBoundingClientRect().bottom, room = window.innerHeight - top;
    var cells = slice(cur.querySelectorAll(".cell.code.sec"));
    for (var i = 0; i < cells.length; i++) {
      var r = cells[i].getBoundingClientRect();
      var visible = Math.min(r.bottom, window.innerHeight) - Math.max(r.top, top);
      if (visible >= Math.min(r.height / 2, room / 3)) return i;
    }
    return cells.length - 1;
  }
  function step(d) {
    if (at < 0) { focusSec(firstVisible()); return; }
    focusSec(Math.max(0, Math.min(secs.length - 1, at + d)));
  }

  // ---- pinning -------------------------------------------------------------------
  function unpin() {
    pinned.forEach(function (el) {
      el.classList.remove("pin");
      var tag = el.querySelector(":scope > .by");
      if (tag) tag.parentNode.removeChild(tag);
    });
    pinned = [];
  }
  function pin(els) {
    els.forEach(function (el) { el.classList.add("pin"); });
    pinned = pinned.concat(els);
  }
  function codeRowsFor(p, fallback) {
    var rows = slice(cur.querySelectorAll('.cell.code .ln[data-p="' + p + '"]'));
    if (!rows.length) {
      var one = cur.querySelector('.cell.code .ln[data-l="' + p + '"]') || fallback;
      if (one) rows = [one];
    }
    return rows;
  }
  function visibleRows(sel) {
    var run = cur.dataset.run || "1";
    return slice(cur.querySelectorAll(sel)).filter(function (el) {
      var r = el.closest(".run");
      return !r || r.dataset.run === run;
    });
  }
  function pinCode(row) {
    var p = row.dataset.p;
    if (!p) { pin([row]); return; }
    var outs = visibleRows('.ol[data-src="' + p + '"], .ol[data-also~="' + p + '"]');
    pin(codeRowsFor(p, row).concat(outs));
  }
  function flash(el) {
    el.classList.remove("flash");
    void el.offsetWidth;
    el.classList.add("flash");
  }
  function onClick(e) {
    // A link to the hash already showing does not fire hashchange; re-apply it.
    var link = e.target.closest('a[href^="#"]');
    if (link) {
      if (link.getAttribute("href") === location.hash) setTimeout(route, 0);
      return;
    }
    if (!cur) return;
    var tag = e.target.closest(".by");
    if (tag) {
      var target = cur.querySelector('.cell.code .ln[data-l="' + tag.dataset.l + '"]');
      if (target) {
        target.scrollIntoView({ block: "center", behavior: "smooth" });
        flash(target);
      }
      return;
    }
    if (e.target.closest("a, button, select, .more")) return;
    if (window.getSelection && String(window.getSelection()).length > 0) return;
    var row = e.target.closest(".ol[data-src], .ln.ps, .dl[data-src]");
    if (!row || !cur.contains(row)) return;
    if (row.closest("summary")) e.preventDefault();
    var was = row.classList.contains("pin");
    unpin();
    if (was) return;
    if (row.classList.contains("ln")) {
      pinCode(row);
      return;
    }
    var srcs = [row.dataset.src].concat((row.dataset.also || "").split(" ").filter(Boolean));
    var code = [];
    srcs.forEach(function (s) { code = code.concat(codeRowsFor(s)); });
    code.forEach(function (c) {
      var fold = c.closest("details");
      if (fold && !fold.open) fold.open = true;
    });
    pin([row].concat(code));
    if (row.dataset.by && code.length) {
      var below = code[0].getBoundingClientRect().top > row.getBoundingClientRect().top;
      var t = doc.createElement("span");
      t.className = "by";
      t.dataset.l = row.dataset.src;
      t.textContent = (below ? "↓" : "↑") + " line " + row.dataset.src + " · " + row.dataset.by +
        (row.dataset.drop ? " · runs when its owner's scope ends" : "");
      row.appendChild(t);
    }
  }

  function toggleDetails() {
    if (!cur) return;
    var d = cur.querySelector("details.why, details.build");
    if (!d) return;
    d.open = !d.open;
    if (d.open) d.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }

  // ---- wiring ------------------------------------------------------------------------
  doc.addEventListener("click", onClick);
  pick.addEventListener("change", function () {
    location.hash = pick.value;
    pick.blur();
  });
  doc.querySelector(".bar .prev").addEventListener("click", function (e) { go(-1); e.currentTarget.blur(); });
  doc.querySelector(".bar .next").addEventListener("click", function (e) { go(1); e.currentTarget.blur(); });
  slice(doc.querySelectorAll(".bar .size button")).forEach(function (b) {
    b.addEventListener("click", function () { setFs(currentFs() + parseInt(b.dataset.fs, 10), true); b.blur(); });
  });
  slice(doc.querySelectorAll(".cmds .tab")).forEach(function (t) {
    t.addEventListener("click", function () { setRun(parseInt(t.dataset.run, 10)); t.blur(); });
  });

  var digits = "", digitsAt = 0;
  doc.addEventListener("keydown", function (e) {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    var t = e.target;
    if (t && /^(SELECT|INPUT|TEXTAREA)$/.test(t.tagName)) return;
    var k = e.key;
    var stepping = cur && secs.length > 0;   // broken files have no sections: let keys scroll
    if (k === " " || k === "j" || k === "ArrowDown" || k === "ArrowRight") {
      if (!stepping) return;
      e.preventDefault();
      step(k === " " && e.shiftKey ? -1 : 1);
    } else if (k === "k" || k === "ArrowUp" || k === "ArrowLeft") {
      if (!stepping) return;
      e.preventDefault();
      step(-1);
    } else if (/^[0-9]$/.test(k)) {
      if (!stepping) return;
      // Two digits typed quickly reach sections 10 and up.
      var now = Date.now();
      var both = now - digitsAt < 700 ? digits + k : "";
      digitsAt = now;
      if (both && secs.indexOf(String(parseInt(both, 10))) >= 0) {
        digits = "";
        focusSec(secs.indexOf(String(parseInt(both, 10))));
      } else {
        digits = k;
        if (secs.indexOf(k) >= 0) focusSec(secs.indexOf(k));
      }
    } else if (k === "Escape") {
      if (pinned.length) unpin(); else unfocus();
    } else if (k === "n") {
      go(1);
    } else if (k === "p") {
      go(-1);
    } else if (k === "+" || k === "=") {
      setFs(currentFs() + 2, true);
    } else if (k === "-" || k === "_") {
      setFs(currentFs() - 2, true);
    } else if (k === "e") {
      toggleDetails();
    } else if (k === "r") {
      cycleRun();
    }
  });

  window.addEventListener("hashchange", route);
  route();
})();
