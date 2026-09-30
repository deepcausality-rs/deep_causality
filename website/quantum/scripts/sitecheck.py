#!/usr/bin/env python3
"""Prose lint and internal-link check for the built site.

Run it after a build: `pnpm check:prose` builds first, then runs this script.
Direct use: `python3 scripts/sitecheck.py [--dist DIR] [--strict] [route ...]`.

Prose rules come from docs/writing_guides (ElementsOfStyle, AiStyleguide) and the
"Voice and vocabulary" section of README.md. Errors fail the run; warnings print
and fail only under --strict.

  error    banned phrase, filler word, transition-word paragraph opener,
           "flagship" or "marketing site", duplicate id, broken link or anchor
  warning  technical jargon outside the pages that define it, more than one em
           dash per 250 words, "not A, but B", a sentence over 45 words, uniform
           sentence length, words glued together across markup

Only the text a reader sees is linted: the content of <main>, plus each page's
<title> and meta description. Text inside <pre>, <script>, <style>, <svg> and any
aria-hidden element is skipped, and inline <code> becomes a placeholder, so
identifiers never trigger a prose rule. The allowlists below record where a term
is meant to appear. A page that yields no prose is an error: it means the layout
lost its <main> and the lint would otherwise pass silently. A redirect stub (a page
whose only content is a meta refresh) is not linted for prose; its internal target
must exist, anchor included.

Exit status: 0 clean, 1 lint errors, 2 usage error (no build, unknown route).
"""

from __future__ import annotations

import argparse
import posixpath
import re
import statistics
import sys
from urllib.parse import unquote
from dataclasses import dataclass
from html.parser import HTMLParser
from pathlib import Path

ERROR = "error"
WARNING = "warning"

# --- Rules -----------------------------------------------------------------

BANNED_PHRASES = [
    "delve", "shed light", "game-changer", "game changer", "unlock", "seamless",
    "robust", "powerful", "leverage", "not only", "arguably", "somewhat",
    "flagship", "marketing site", "cutting-edge", "state-of-the-art",
    "revolutionary", "paradigm",
]
# Whole words only: "every" must not match "very".
BANNED_WORDS = ["very", "really"]

# A banned phrase that is part of a verbatim quotation may appear on the page
# that quotes it. Nothing on this site needs one today.
BANNED_ALLOW: dict[str, set[str]] = {}

OPENERS = ("additionally", "furthermore", "moreover", "in addition,")

# Terms kept out of ordinary prose. The value lists the routes that define the
# term, where it may appear (each is glossed in the sentence that uses it). A route
# ending in `*` covers every route under that prefix.
JARGON = {
    "causaloid": set(),
    "predicate": set(),
    "orthomodular": {"/checks/", "/proof/"},
    "kraus": {"/checks/", "/proof/", "/examples/*"},
    "choi–jamiołkowski": {"/how-it-works/", "/checks/", "/proof/", "/start/", "/boundaries/", "/examples/*"},
    "monad": {"/how-it-works/", "/start/", "/boundaries/", "/examples/*"},
    "counterfactual": {"/examples/*"},
}

MAX_SENTENCE_WORDS = 45
MIN_SENTENCES_FOR_RHYTHM = 12
MIN_SENTENCE_STDEV = 6.0
EM_DASHES_PER_250_WORDS = 1.0

NOT_A_BUT_B = re.compile(r"\b(?:is|are|was|were) not [^.;:]{3,60}, (?:but|it is|it's)\b", re.I)
GLUED_COMMA = re.compile(r"[a-z][,;][A-Za-z0-9]")
GLUED_PERIOD = re.compile(r"(?<![.\w])[a-z]{3,}\.[A-Z][a-z]{2,}")

# --- HTML extraction -------------------------------------------------------

VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
        "param", "source", "track", "wbr"}
SKIP = {"script", "style", "noscript", "svg", "pre"}
BREAKS = {"p", "li", "dd", "dt", "td", "th", "tr", "figcaption", "h1", "h2", "h3", "h4",
          "summary", "blockquote", "div", "section", "header", "footer", "ul", "ol",
          "table", "thead", "tbody", "dl", "article", "nav", "figure", "details", "main"}


class Page(HTMLParser):
    """Collects the linted text blocks, every id and every internal href of a page."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.paras: list[str] = []
        self.meta: list[str] = []
        self.ids: list[str] = []
        self.hrefs: list[str] = []
        self._buf: list[str] = []
        self._in_main = 0
        self._in_title = False
        self._skip_tag: str | None = None
        self._skip_depth = 0
        self._code = 0

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if a.get("id"):
            self.ids.append(a["id"])
        if tag == "meta" and (a.get("name") == "description" or a.get("property") == "og:description"):
            if a.get("content"):
                self.meta.append(re.sub(r"\s+", " ", a["content"]).strip())
        if tag == "a" and a.get("href"):
            self.hrefs.append(a["href"])
        if self._skip_tag:
            if tag == self._skip_tag and tag not in VOID:
                self._skip_depth += 1
            return
        if tag in SKIP or a.get("aria-hidden") == "true":
            # A skipped block ends the text block before it; inline decorations do not.
            if tag == "pre" or tag in BREAKS:
                self._flush()
            if tag not in VOID:
                self._skip_tag, self._skip_depth = tag, 1
            return
        if tag == "title":
            self._in_title = True
        if tag == "main":
            self._in_main += 1
        if tag == "code":
            if self._code == 0 and self._in_main:
                self._buf.append(" CODE ")
            self._code += 1
        if tag in BREAKS:
            self._flush()

    def handle_endtag(self, tag):
        if self._skip_tag:
            if tag == self._skip_tag:
                self._skip_depth -= 1
                if self._skip_depth == 0:
                    self._skip_tag = None
            return
        if tag == "title":
            self._in_title = False
        if tag == "code":
            self._code = max(0, self._code - 1)
        if tag == "main":
            self._flush()
            self._in_main = max(0, self._in_main - 1)
        if tag in BREAKS:
            self._flush()

    def handle_data(self, data):
        if self._skip_tag:
            return
        if self._in_title:
            self.meta.append(re.sub(r"\s+", " ", data).strip())
            return
        if not self._in_main or self._code:
            return
        self._buf.append(data)

    def _flush(self):
        text = re.sub(r"\s+", " ", "".join(self._buf)).strip()
        if text:
            self.paras.append(text)
        self._buf = []

    def close(self):
        super().close()
        self._flush()

    @property
    def truncated(self) -> bool:
        """True when an unclosed skip or <code> swallowed the rest of the page."""
        return bool(self._skip_tag) or self._code > 0


def parse(html: str) -> Page:
    page = Page()
    page.feed(html)
    page.close()
    return page


# --- Prose checks ----------------------------------------------------------

@dataclass(frozen=True)
class Finding:
    level: str
    message: str


def sentences(text: str) -> list[str]:
    return [s for s in re.split(r"(?<=[.!?])\s+(?=[A-Z“\"(\[])", text) if s.strip()]


def at_home(route: str, homes: set[str]) -> bool:
    """True when `route` is one of `homes`, or lies under a `homes` entry ending in `*`."""
    return any(route.startswith(h[:-1]) if h.endswith("*") else route == h for h in homes)


def lint_prose(route: str, paras: list[str]) -> list[Finding]:
    out: list[Finding] = []
    text = " ".join(paras)
    low = text.lower()
    words = len(text.split())

    for phrase in BANNED_PHRASES:
        n = low.count(phrase)
        if n and route not in BANNED_ALLOW.get(phrase, ()):
            out.append(Finding(ERROR, f'banned "{phrase}" x{n}'))
    for word in BANNED_WORDS:
        n = len(re.findall(rf"\b{word}\b", low))
        if n:
            out.append(Finding(ERROR, f'filler word "{word}" x{n}'))
    for para in paras:
        if para.lower().startswith(OPENERS):
            out.append(Finding(ERROR, f"paragraph opens with a transition word: {para[:50]!r}"))

    for term, homes in JARGON.items():
        n = len(re.findall(rf"(?<![\w-]){re.escape(term)}(?:s|es)?(?![\w-])", low))
        if n and not at_home(route, homes):
            out.append(Finding(WARNING, f'jargon "{term}" x{n}'))

    dashes = text.count("—")
    if words and dashes * 250 / words > EM_DASHES_PER_250_WORDS:
        out.append(Finding(WARNING, f"em dashes: {dashes} in {words} words"))

    for para in paras:
        m = NOT_A_BUT_B.search(para)
        if m:
            out.append(Finding(WARNING, f'"not A, but B": {m.group(0)[:70]!r}'))
        g = GLUED_COMMA.search(para) or GLUED_PERIOD.search(para)
        if g:
            out.append(Finding(WARNING, f"words glued across markup: {g.group(0)!r}"))

    # Rhythm and length apply to running prose: a block that ends in a full stop.
    prose = [s for para in paras if para.endswith((".", "!", "?")) for s in sentences(para)]
    lengths = [len(s.split()) for s in prose if len(s.split()) > 2]
    if lengths and max(lengths) > MAX_SENTENCE_WORDS:
        out.append(Finding(WARNING, f"longest sentence has {max(lengths)} words"))
    if len(lengths) >= MIN_SENTENCES_FOR_RHYTHM and statistics.pstdev(lengths) < MIN_SENTENCE_STDEV:
        out.append(Finding(
            WARNING,
            f"sentence length too uniform: mean {statistics.mean(lengths):.1f}, "
            f"sd {statistics.pstdev(lengths):.1f} (want sd >= {MIN_SENTENCE_STDEV:g})",
        ))
    return out


def lint_meta(route: str, strings: list[str]) -> list[Finding]:
    """Banned phrases and filler words in the <title> and meta descriptions."""
    out: list[Finding] = []
    low = " ".join(strings).lower()
    for phrase in BANNED_PHRASES:
        if phrase in low and route not in BANNED_ALLOW.get(phrase, ()):
            out.append(Finding(ERROR, f'banned "{phrase}" in the title or description'))
    for word in BANNED_WORDS:
        if re.search(rf"\b{word}\b", low):
            out.append(Finding(ERROR, f'filler word "{word}" in the title or description'))
    return out


# --- Links -----------------------------------------------------------------

REDIRECT = re.compile(r'<meta[^>]+http-equiv="refresh"[^>]+content="\d+;\s*url=([^"]+)"', re.I)


def redirect_target(html: str) -> str | None:
    """The target of a redirect stub, or None for an ordinary page."""
    m = REDIRECT.search(html)
    return m.group(1) if m else None


def route_of(dist: Path, file: Path) -> str:
    rel = file.relative_to(dist).as_posix()
    if rel == "index.html":
        return "/"
    if rel.endswith("/index.html"):
        return "/" + rel[: -len("index.html")]
    return "/" + rel


def resolve(route: str, href: str) -> tuple[str, str] | None:
    """(path, fragment) of an internal href, or None for an external or non-page link."""
    if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", href) or href.startswith("//"):
        return None
    path, _, fragment = href.partition("#")
    path = unquote(path.partition("?")[0])
    fragment = unquote(fragment)
    if not path:
        return route, fragment
    if not path.startswith("/"):
        base = route if route.endswith("/") else posixpath.dirname(route).rstrip("/") + "/"
        directory = path.endswith("/")
        path = posixpath.normpath(posixpath.join(base, path))
        if directory and not path.endswith("/"):
            path += "/"
    if path.endswith("/index.html"):
        path = path[: -len("index.html")]
    return path, fragment


def lint_links(route: str, page: Page, ids_by_route: dict[str, set[str]], static: set[str]) -> list[Finding]:
    out: list[Finding] = []
    seen: set[str] = set()
    for i in page.ids:
        if i in seen:
            out.append(Finding(ERROR, f'duplicate id "{i}"'))
        seen.add(i)
    for href in page.hrefs:
        target = resolve(route, href)
        if target is None:
            continue
        path, fragment = target
        page_key = path if path.endswith("/") or path == "/" else path + "/"
        if path in ids_by_route or page_key in ids_by_route:
            ids = ids_by_route.get(path, ids_by_route.get(page_key, set()))
            # "#top" is the document start by HTML spec, whether or not an id exists.
            if fragment and fragment.lower() != "top" and fragment not in ids:
                out.append(Finding(ERROR, f"broken anchor {href}"))
        elif path.lstrip("/") in static:
            continue
        else:
            out.append(Finding(ERROR, f"broken link {href}"))
    return out


# --- Driver ----------------------------------------------------------------

def normalize_route(route: str) -> str:
    route = "/" + route.strip("/")
    return route if route == "/" or route.endswith(".html") else route + "/"


def run(dist: Path, routes: list[str], strict: bool, out=None) -> int:
    out = out or sys.stdout
    files = sorted(dist.rglob("index.html")) + sorted(dist.glob("404.html"))
    if not files:
        print(f"no built pages under {dist}; run `pnpm build` first", file=out)
        return 2

    texts = {route_of(dist, f): f.read_text(encoding="utf-8", errors="replace") for f in files}
    redirects = {r: t for r, h in texts.items() if (t := redirect_target(h))}
    pages = {r: parse(h) for r, h in texts.items() if r not in redirects}
    ids_by_route = {r: set(p.ids) for r, p in pages.items()}
    static = {p.relative_to(dist).as_posix() for p in dist.rglob("*") if p.is_file()}

    wanted = {normalize_route(r) for r in routes}
    unknown = wanted - set(pages) - set(redirects)
    if unknown:
        print(f"unknown route(s): {', '.join(sorted(unknown))}", file=out)
        return 2

    errors = warnings = 0
    for route, target in redirects.items():
        if wanted and route not in wanted:
            continue
        goal = resolve(route, target)
        if goal is None:
            print(f"{route}  redirect to {target}", file=out)
            continue
        path, fragment = goal
        key = path if path.endswith("/") or path == "/" else path + "/"
        found = key in ids_by_route or path in ids_by_route
        ids = ids_by_route.get(path, ids_by_route.get(key, set()))
        if not found or (fragment and fragment.lower() != "top" and fragment not in ids):
            print(f"\n{route}  redirect", file=out)
            print(f"   {ERROR:7} broken redirect target {target}", file=out)
            errors += 1
        else:
            print(f"{route}  redirect to {target}", file=out)
    for route, page in pages.items():
        if wanted and route not in wanted:
            continue
        words = sum(len(p.split()) for p in page.paras)
        findings = lint_prose(route, page.paras) + lint_meta(route, page.meta)
        findings += lint_links(route, page, ids_by_route, static)
        if words == 0:
            findings.insert(0, Finding(ERROR, "no prose extracted: the page has no <main> text, so nothing was linted"))
        if page.truncated:
            findings.append(Finding(WARNING, "unclosed <code> or aria-hidden markup: text after it was not linted"))
        if not findings:
            print(f"{route}  ok ({words} words)", file=out)
            continue
        print(f"\n{route}  ({words} words)", file=out)
        for f in findings:
            print(f"   {f.level:7} {f.message}", file=out)
            errors += f.level == ERROR
            warnings += f.level == WARNING
    print(f"\n{errors} error(s), {warnings} warning(s)", file=out)
    return 1 if errors or (strict and warnings) else 0


def default_dist() -> Path:
    return Path(__file__).resolve().parent.parent / "dist"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("routes", nargs="*", help="routes to check, e.g. /how-it-works/ (default: all)")
    parser.add_argument("--dist", type=Path, default=default_dist(),
                        help="build output directory (default: ../dist)")
    parser.add_argument("--strict", action="store_true", help="fail on warnings too")
    args = parser.parse_args(argv)
    return run(args.dist, args.routes, args.strict)


if __name__ == "__main__":
    sys.exit(main())
