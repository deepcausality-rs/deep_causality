"""Tests for sitecheck.py: each rule fires on a bad fixture and stays quiet on a good one.

Run with `pnpm check:prose:test`.
"""

import io
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import sitecheck as sc  # noqa: E402


def page(body: str, head: str = "") -> str:
    return f"<html><head>{head}</head><body><header><a href='/'>Home</a></header><main>{body}</main></body></html>"


def prose(html: str, route: str = "/x/") -> list:
    return sc.lint_prose(route, sc.parse(page(html)).paras)


def messages(findings, level=None) -> str:
    return " | ".join(f.message for f in findings if level is None or f.level == level)


class ParseTests(unittest.TestCase):
    def test_only_main_text_and_skipped_regions_are_left_out(self):
        html = (
            "<html><body><nav>Nav text</nav><main><p>Kept.</p><pre>code block</pre>"
            "<script>var x = 1;</script><style>p{}</style><svg><text>svg text</text></svg>"
            "<span aria-hidden='true'>→ hidden</span><p>Also kept.</p></main></body></html>"
        )
        self.assertEqual(sc.parse(html).paras, ["Kept.", "Also kept."])

    def test_nested_skipped_elements_close_at_the_right_tag(self):
        html = "<main><div aria-hidden='true'><div>inner</div> still hidden</div><p>after</p></main>"
        self.assertEqual(sc.parse(html).paras, ["after"])

    def test_inline_code_becomes_a_placeholder(self):
        paras = sc.parse("<main><p>Call <code>causaloid(x)</code> now.</p></main>").paras
        self.assertEqual(paras, ["Call CODE now."])

    def test_ids_and_hrefs_are_collected_from_the_whole_page(self):
        p = sc.parse("<a href='/a/'>a</a><main id='m'><a href='/b/#c'>b</a></main>")
        self.assertEqual(p.ids, ["m"])
        self.assertEqual(p.hrefs, ["/a/", "/b/#c"])

    def test_void_tag_does_not_hang_a_skip(self):
        html = "<main><span aria-hidden='true'><br>text</span><p>ok</p></main>"
        self.assertEqual(sc.parse(html).paras, ["ok"])

    def test_text_before_the_first_block_is_kept(self):
        self.assertEqual(sc.parse("<main>Bare text</main>").paras, ["Bare text"])

    def test_a_skipped_block_ends_the_text_block_before_and_after_it(self):
        cases = {
            "<main><div>Run:<pre>x</pre>Additionally, read.</div></main>": ["Run:", "Additionally, read."],
            "<main><li>Install it<pre>x</pre>Moreover, run it</li></main>": ["Install it", "Moreover, run it"],
            "<main><div>Intro<div aria-hidden='true'>chart</div>Very fast</div></main>": ["Intro", "Very fast"],
        }
        for html, expected in cases.items():
            self.assertEqual(sc.parse(html).paras, expected, html)

    def test_an_inline_decoration_does_not_split_a_sentence(self):
        html = "<main><p>The arrow <span aria-hidden='true'>→</span> points on <svg><g/></svg> and ends.</p></main>"
        self.assertEqual(sc.parse(html).paras, ["The arrow points on and ends."])

    def test_title_and_meta_descriptions_are_collected_apart_from_the_body(self):
        html = ("<html><head><title>Page | Site</title><meta name='description' content=' A  plain  line. '>"
                "<meta property='og:description' content='Share text.'></head><body><main><p>Body.</p></main></body></html>")
        p = sc.parse(html)
        self.assertEqual(p.meta, ["Page | Site", "A plain line.", "Share text."])
        self.assertEqual(p.paras, ["Body."])

    def test_truncation_is_reported_for_unclosed_code_or_hidden_markup(self):
        self.assertTrue(sc.parse("<main><p>Call <code>run(x) now.</p><p>More.</p></main>").truncated)
        self.assertTrue(sc.parse("<main><p>Ok.</p><div aria-hidden='true'>never closed</main>").truncated)
        self.assertFalse(sc.parse("<main><p>Fine <code>x</code>.</p></main>").truncated)


class ProseTests(unittest.TestCase):
    def test_clean_prose_has_no_findings(self):
        self.assertEqual(prose("<p>The run pauses. It returns the state, and every branch continues from it.</p>"), [])

    def test_banned_phrases_are_errors(self):
        f = prose("<p>We delve into a seamless, robust design.</p>")
        for word in ("delve", "seamless", "robust"):
            self.assertIn(word, messages(f, sc.ERROR))

    def test_every_does_not_match_very(self):
        self.assertEqual(prose("<p>Every branch runs.</p>"), [])

    def test_filler_words_match_whole_words_only(self):
        self.assertIn("very", messages(prose("<p>It is very fast.</p>"), sc.ERROR))
        self.assertIn("really", messages(prose("<p>It is really fast.</p>"), sc.ERROR))
        self.assertEqual(prose("<p>The verify step and the reality check.</p>"), [])

    def test_banned_phrase_is_allowed_where_a_quotation_needs_it(self):
        html = "<p>Revolutionary Computational Aerosciences is a report title.</p>"
        with mock.patch.dict(sc.BANNED_ALLOW, {"revolutionary": {"/why/"}}):
            self.assertIn("revolutionary", messages(prose(html, "/x/"), sc.ERROR))
            self.assertEqual(messages(prose(html, "/why/"), sc.ERROR), "")

    def test_transition_openers_are_errors(self):
        for opener in ("Additionally", "Furthermore", "Moreover"):
            self.assertIn("transition word", messages(prose(f"<p>{opener} the run ends.</p>"), sc.ERROR))
        self.assertEqual(prose("<p>The run ends. Additionally it logs.</p>"), [])

    def test_jargon_warns_outside_its_home_and_not_inside(self):
        html = "<p>The causaloid fires and the Choi–Jamiołkowski operator stays small.</p>"
        outside = messages(prose(html, "/x/"), sc.WARNING)
        self.assertIn("causaloid", outside)
        self.assertIn("choi–jamiołkowski", outside)
        inside = messages(prose(html, "/checks/"), sc.WARNING)
        self.assertIn("causaloid", inside)
        self.assertNotIn("choi–jamiołkowski", inside)

    def test_a_prefix_home_covers_every_route_under_it(self):
        html = "<p>The run is a counterfactual.</p>"
        self.assertIn("counterfactual", messages(prose(html, "/checks/"), sc.WARNING))
        self.assertEqual(messages(prose(html, "/examples/quantum-counterfactual/"), sc.WARNING), "")
        self.assertTrue(sc.at_home("/a/b/", {"/a/*"}))
        self.assertFalse(sc.at_home("/a/", {"/a/b/"}))

    def test_jargon_matches_plurals(self):
        self.assertIn("causaloid", messages(prose("<p>Both causaloids ran.</p>"), sc.WARNING))

    def test_jargon_in_inline_code_is_ignored(self):
        self.assertEqual(prose("<p>Pass <code>causaloid</code> to the builder.</p>"), [])

    def test_hyphenated_words_do_not_match_a_jargon_term(self):
        self.assertEqual(prose("<p>The pre-causaloid-free run is fine.</p>"), [])

    def test_em_dash_density(self):
        many = "<p>" + " ".join(["one — two"] * 5) + " word" * 40 + ".</p>"
        self.assertIn("em dashes", messages(prose(many), sc.WARNING))
        few = "<p>" + "word " * 300 + "one — two.</p>"
        self.assertNotIn("em dashes", messages(prose(few)))

    def test_not_a_but_b(self):
        self.assertIn("not A, but B", messages(prose("<p>This is not a bug, but a feature.</p>"), sc.WARNING))

    def test_glued_words(self):
        self.assertIn("glued", messages(prose("<p>Six branches rerun,12 steps.</p>"), sc.WARNING))
        self.assertIn("glued", messages(prose("<p>It ends with rerun.Read the note.</p>"), sc.WARNING))
        self.assertEqual(prose("<p>The table has 1,386 steps and 0.5 degrees.</p>"), [])

    def test_long_sentence_warns_and_fragments_are_ignored(self):
        long_sentence = " ".join(["word"] * 50) + "."
        self.assertIn("longest sentence", messages(prose(f"<p>{long_sentence}</p>"), sc.WARNING))
        fragment = " ".join(["cell"] * 60)
        self.assertEqual(prose(f"<p>{fragment}</p>"), [])

    def test_uniform_rhythm_needs_enough_sentences(self):
        uniform = "".join(f"<p>{' '.join(['word'] * 12)}.</p>" for _ in range(14))
        self.assertIn("too uniform", messages(prose(uniform), sc.WARNING))
        varied = "".join(f"<p>{' '.join(['word'] * n)}.</p>" for n in (3, 30, 5, 22, 9, 35, 4, 18, 7, 26, 3, 31))
        self.assertNotIn("too uniform", messages(prose(varied)))
        short = "".join(f"<p>{' '.join(['word'] * 12)}.</p>" for _ in range(5))
        self.assertNotIn("too uniform", messages(prose(short)))


class MetaTests(unittest.TestCase):
    def test_banned_phrases_and_filler_in_title_or_description_are_errors(self):
        f = sc.lint_meta("/x/", ["A seamless engine", "It is really fast."])
        self.assertIn("seamless", messages(f, sc.ERROR))
        self.assertIn("really", messages(f, sc.ERROR))

    def test_clean_meta_and_the_allowlist(self):
        self.assertEqual(sc.lint_meta("/x/", ["Find the cause | Site"]), [])
        with mock.patch.dict(sc.BANNED_ALLOW, {"revolutionary": {"/why/"}}):
            self.assertEqual(sc.lint_meta("/why/", ["Revolutionary Computational Aerosciences"]), [])
            self.assertNotEqual(sc.lint_meta("/x/", ["Revolutionary Computational Aerosciences"]), [])


class LinkTests(unittest.TestCase):
    IDS = {"/": {"top"}, "/a/": {"sec"}, "/a/b/": set()}
    STATIC = {"brief.pdf", "img/x.png"}

    def links(self, html: str, route: str = "/a/"):
        return sc.lint_links(route, sc.parse(html), self.IDS, self.STATIC)

    def test_valid_links_and_anchors_pass(self):
        html = "<a href='/'>h</a><a href='/a/#sec'>s</a><a href='/a'>no slash</a><a href='#sec'>self</a><div id='sec'></div>"
        self.assertEqual(self.links(html), [])

    def test_broken_link_and_anchor_are_errors(self):
        f = self.links("<a href='/missing/'>x</a><a href='/a/#nope'>y</a>")
        self.assertEqual(messages(f), "broken link /missing/ | broken anchor /a/#nope")

    def test_same_page_anchor_is_checked_against_the_page_route(self):
        self.assertIn("broken anchor", messages(self.links("<a href='#nope'>x</a>")))

    def test_external_and_non_page_links_are_skipped(self):
        html = "<a href='https://x.dev/y'>e</a><a href='mailto:a@b.c'>m</a><a href='//cdn/x'>p</a><a href='tel:1'>t</a>"
        self.assertEqual(self.links(html), [])

    def test_static_assets_resolve(self):
        self.assertEqual(self.links("<a href='/brief.pdf'>pdf</a>"), [])

    def test_relative_links_resolve_against_the_route(self):
        self.assertEqual(self.links("<a href='b/'>b</a>", "/a/"), [])
        self.assertEqual(self.links("<a href='../'>up</a>", "/a/b/"), [])
        self.assertIn("broken link", messages(self.links("<a href='../a/'>a/a</a>", "/a/b/")))
        self.assertIn("broken link", messages(self.links("<a href='zzz/'>z</a>", "/a/")))

    def test_cross_page_anchors_are_checked_against_the_target_page(self):
        self.assertEqual(self.links("<a href='/#top'>t</a>", "/a/b/"), [])
        self.assertEqual(messages(self.links("<a href='/#nope'>t</a>", "/a/b/")), "broken anchor /#nope")
        self.assertEqual(messages(self.links("<a href='/a/b/#sec'>s</a>", "/a/")), "broken anchor /a/b/#sec")
        self.assertEqual(self.links("<a href='/a/#sec'>s</a>", "/a/b/"), [])

    def test_the_top_fragment_is_valid_without_an_id(self):
        self.assertEqual(self.links("<a href='#top'>t</a><a href='#TOP'>t</a><a href='#'>t</a>"), [])

    def test_percent_encoded_paths_and_fragments_are_decoded(self):
        ids = {"/a/": {"résumé"}, "/x y/": set()}
        p = sc.parse("<a href='#r%C3%A9sum%C3%A9'>a</a><a href='/x%20y/'>b</a>")
        self.assertEqual(sc.lint_links("/a/", p, ids, set()), [])

    def test_index_html_links_resolve_to_the_directory_and_check_anchors(self):
        self.assertEqual(self.links("<a href='/a/index.html#sec'>s</a>"), [])
        self.assertEqual(messages(self.links("<a href='/a/index.html#nope'>s</a>")), "broken anchor /a/index.html#nope")
        self.assertEqual(self.links("<a href='/index.html'>home</a>"), [])

    def test_relative_links_from_the_404_page_resolve_from_the_root(self):
        self.assertEqual(sc.resolve("/404.html", "a/"), ("/a/", ""))
        self.assertEqual(sc.resolve("/404.html", "brief.pdf"), ("/brief.pdf", ""))

    def test_query_strings_are_ignored(self):
        self.assertEqual(self.links("<a href='/a/?x=1#sec'>q</a>"), [])

    def test_duplicate_ids_are_errors(self):
        f = self.links("<div id='d'></div><p id='d'></p>")
        self.assertEqual(messages(f, sc.ERROR), 'duplicate id "d"')


class DriverTests(unittest.TestCase):
    def build(self, pages: dict[str, str], extra: dict[str, str] | None = None) -> Path:
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        root = Path(tmp.name)
        for rel, html in pages.items():
            path = root / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(html, encoding="utf-8")
        for rel, text in (extra or {}).items():
            (root / rel).write_text(text, encoding="utf-8")
        return root

    def go(self, dist: Path, routes=(), strict=False):
        out = io.StringIO()
        return sc.run(dist, list(routes), strict, out), out.getvalue()

    def test_route_of(self):
        d = Path("/d")
        self.assertEqual(sc.route_of(d, d / "index.html"), "/")
        self.assertEqual(sc.route_of(d, d / "a/index.html"), "/a/")
        self.assertEqual(sc.route_of(d, d / "a/b/index.html"), "/a/b/")
        self.assertEqual(sc.route_of(d, d / "404.html"), "/404.html")

    def test_clean_site_exits_zero(self):
        dist = self.build({"index.html": page("<p>Fine.</p><a href='/a/'>a</a>"), "a/index.html": page("<p>Also fine.</p>")})
        code, out = self.go(dist)
        self.assertEqual(code, 0)
        self.assertIn("0 error(s), 0 warning(s)", out)

    def test_error_exits_one(self):
        dist = self.build({"index.html": page("<p>A seamless run.</p>")})
        code, out = self.go(dist)
        self.assertEqual(code, 1)
        self.assertIn("seamless", out)

    def test_warning_exits_zero_unless_strict(self):
        dist = self.build({"index.html": page("<p>The predicate fires.</p>")})
        self.assertEqual(self.go(dist)[0], 0)
        self.assertEqual(self.go(dist, strict=True)[0], 1)

    def test_broken_link_across_pages_is_reported(self):
        dist = self.build({"index.html": page("<a href='/gone/'>x</a>")})
        code, out = self.go(dist)
        self.assertEqual(code, 1)
        self.assertIn("broken link /gone/", out)

    def test_route_filter_checks_only_the_named_route(self):
        dist = self.build({"index.html": page("<p>A seamless run.</p>"), "a/index.html": page("<p>Fine.</p>")})
        self.assertEqual(self.go(dist, ["/a"])[0], 0)
        self.assertEqual(self.go(dist, ["/"])[0], 1)

    def test_usage_errors_exit_two(self):
        dist = self.build({"index.html": page("<p>Fine.</p>")})
        code, out = self.go(dist, ["/nope/"])
        self.assertEqual((code, "unknown route" in out), (2, True))
        empty = self.build({}, {"readme.txt": "x"})
        code, out = self.go(empty)
        self.assertEqual((code, "pnpm build" in out), (2, True))

    def test_a_page_without_prose_is_an_error(self):
        no_main = "<html><body><div role='main'><p>A seamless design.</p></div></body></html>"
        code, out = self.go(self.build({"index.html": no_main}))
        self.assertEqual(code, 1)
        self.assertIn("no prose extracted", out)
        code, out = self.go(self.build({"index.html": page("")}))
        self.assertEqual((code, "no prose extracted" in out), (1, True))

    def test_a_redirect_stub_is_not_linted_for_prose_and_its_target_must_exist(self):
        stub = "<!doctype html><title>Redirecting to: /a/</title><meta http-equiv=\"refresh\" content=\"0;url=/a/\"><body><a href=\"/a/\">go</a></body>"
        dist = self.build({"index.html": page("<p>Fine.</p>"), "a/index.html": page("<p>Also fine.</p>"), "old/index.html": stub})
        code, out = self.go(dist)
        self.assertEqual(code, 0)
        self.assertIn("/old/  redirect to /a/", out)
        self.assertNotIn("no prose extracted", out)

    def test_a_redirect_to_a_missing_page_or_anchor_is_an_error(self):
        def stub(target):
            return f"<title>r</title><meta http-equiv=\"refresh\" content=\"0;url={target}\">"
        base = {"index.html": page("<p>Fine.</p>"), "a/index.html": page("<p id='ok'>Also fine.</p>")}
        code, out = self.go(self.build({**base, "old/index.html": stub("/gone/")}))
        self.assertEqual(code, 1)
        self.assertIn("broken redirect target /gone/", out)
        code, out = self.go(self.build({**base, "old/index.html": stub("/a/#missing")}))
        self.assertEqual(code, 1)
        code, out = self.go(self.build({**base, "old/index.html": stub("/a/#ok")}))
        self.assertEqual(code, 0)

    def test_a_redirect_to_an_external_site_is_accepted(self):
        stub = "<title>r</title><meta http-equiv=\"refresh\" content=\"0;url=https://docs.rs/x\">"
        code, out = self.go(self.build({"index.html": page("<p>Fine.</p>"), "old/index.html": stub}))
        self.assertEqual(code, 0)
        self.assertIn("redirect to https://docs.rs/x", out)

    def test_truncated_markup_warns_and_fails_only_under_strict(self):
        dist = self.build({"index.html": page("<p>Call <code>run(x) now.</p><p>Fine.</p>")})
        code, out = self.go(dist)
        self.assertEqual((code, "unclosed" in out), (0, True))
        self.assertEqual(self.go(dist, strict=True)[0], 1)

    def test_meta_description_errors_fail_the_run(self):
        html = "<html><head><title>T</title><meta name='description' content='A seamless engine.'></head><body><main><p>Fine.</p></main></body></html>"
        code, out = self.go(self.build({"index.html": html}))
        self.assertEqual((code, "title or description" in out), (1, True))

    def test_the_same_id_on_two_pages_is_not_a_duplicate(self):
        dist = self.build({"index.html": page("<p id='x'>Fine.</p>"), "a/index.html": page("<p id='x'>Fine.</p>")})
        self.assertEqual(self.go(dist)[0], 0)

    def test_cross_page_anchor_between_real_files(self):
        home = page("<p>Fine.</p><a href='/a/#sec'>ok</a><a href='/a/#nope'>bad</a>")
        dist = self.build({"index.html": home, "a/index.html": page("<p id='sec'>Fine.</p>")})
        code, out = self.go(dist)
        self.assertEqual(code, 1)
        self.assertIn("broken anchor /a/#nope", out)
        self.assertNotIn("broken anchor /a/#sec", out)

    def test_non_utf8_files_are_read_with_replacement(self):
        dist = self.build({"index.html": page("<p>Fine.</p>")})
        (dist / "a").mkdir()
        (dist / "a" / "index.html").write_bytes(b"<html><body><main><p>caf\xe9 fine.</p></main></body></html>")
        self.assertEqual(self.go(dist)[0], 0)

    def test_404_page_is_checked(self):
        dist = self.build({"index.html": page("<p>Fine.</p>"), "404.html": page("<p>A seamless page.</p>")})
        self.assertEqual(self.go(dist)[0], 1)

    def test_static_files_count_as_link_targets(self):
        dist = self.build({"index.html": page("<a href='/brief.pdf'>pdf</a>")}, {"brief.pdf": "%PDF"})
        self.assertEqual(self.go(dist)[0], 0)

    def main_code(self, argv):
        old = sys.stdout
        sys.stdout = io.StringIO()
        try:
            return sc.main(argv)
        finally:
            sys.stdout = old

    def test_main_reads_the_dist_argument_and_the_route_and_strict_flags(self):
        bad = self.build({"index.html": page("<p>A seamless run.</p>"), "a/index.html": page("<p>Fine.</p>")})
        self.assertEqual(self.main_code(["--dist", str(bad)]), 1)
        self.assertEqual(self.main_code(["--dist", str(bad), "/a/"]), 0)
        warn = self.build({"index.html": page("<p>The predicate fires.</p>")})
        self.assertEqual(self.main_code(["--dist", str(warn)]), 0)
        self.assertEqual(self.main_code(["--dist", str(warn), "--strict"]), 1)
        clean = self.build({"index.html": page("<p>Fine.</p>")})
        self.assertEqual(self.main_code(["--dist", str(clean), "--strict"]), 0)

    def test_the_default_dist_sits_beside_the_scripts_directory(self):
        self.assertEqual(sc.default_dist(), Path(sc.__file__).resolve().parent.parent / "dist")
        self.assertEqual(sc.default_dist().name, "dist")

if __name__ == "__main__":
    unittest.main()
