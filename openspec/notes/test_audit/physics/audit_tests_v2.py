#!/usr/bin/env python3
"""Find test designs that plausible wrong implementations can survive.

This scanner implements the risk-first methodology described in Dan Luu's
"How well do agents use test/verification techniques?".  It reports syntactic
candidates for semantic review; no category is a defect count.

The default target is deliberately the physics crate.  Pass --root to audit a
different test tree explicitly.
"""

from __future__ import annotations

import argparse
import json
import re
from collections import defaultdict, deque
from dataclasses import dataclass
from pathlib import Path


DEFAULT_ROOT = Path("deep_causality_physics/tests")
DEFAULT_JSON = Path("/tmp/physics_test_audit_v2.json")

ASSERT_NAME = re.compile(r"\b(assert|assert_eq|assert_ne|debug_assert|debug_assert_eq|debug_assert_ne)!")
ARITHMETIC = re.compile(
    r"[+\-*/]|\.(?:sqrt|cbrt|powi|powf|ln|log|log10|exp|sin|cos|tan|asin|acos|atan|abs|hypot)\s*\("
)
PROVENANCE = re.compile(
    r"(?:NIST|CODATA|reference|published|textbook|handbook|oracle|closed[- ]form|"
    r"by hand|bisection|analytic|literature|Appendix|ISBN|doi|et al|scipy|numpy|"
    r"mpmath|Wolfram|golden|high[- ]precision|independent|identity|invariant|"
    r"conservation|dimensional|scaling|20\d\d\))",
    re.IGNORECASE,
)
WEAK_PREDICATE = re.compile(
    r"\.(?:is_ok|is_some|is_finite|is_nan|is_sign_positive|is_sign_negative)\s*\(\s*\)"
)
ERROR_PREDICATE = re.compile(
    r"\.(?:is_err|is_none)\s*\(\s*\)|unwrap_err|expect_err|matches!"
)
RANDOM_SOURCE = re.compile(r"\b(?:random|rand|rng|sample|fuzz|proptest|quickcheck)\b", re.IGNORECASE)
DEGENERATE_PURPOSE = re.compile(
    r"(?:zero|one|identity|default|empty|unit|basis|orthogonal|boundary|limit|constant|uniform)",
    re.IGNORECASE,
)


@dataclass(frozen=True)
class TestCase:
    path: Path
    name: str
    line: int
    attributes: str
    body: str
    context: str


def mask_non_code(source: str) -> str:
    """Replace comments and string/character contents while preserving offsets."""
    out = list(source)
    i = 0
    state = "code"
    while i < len(source):
        pair = source[i : i + 2]
        ch = source[i]
        if state == "code":
            if pair == "//":
                out[i] = out[i + 1] = " "
                i += 2
                state = "line-comment"
                continue
            if pair == "/*":
                out[i] = out[i + 1] = " "
                i += 2
                state = "block-comment"
                continue
            if ch == '"':
                out[i] = " "
                i += 1
                state = "string"
                continue
            if ch == "'" and i + 2 < len(source) and source[i + 2] == "'":
                out[i] = out[i + 2] = " "
                out[i + 1] = " "
                i += 3
                continue
        elif state == "line-comment":
            if ch == "\n":
                state = "code"
            else:
                out[i] = " "
        elif state == "block-comment":
            if pair == "*/":
                out[i] = out[i + 1] = " "
                i += 2
                state = "code"
                continue
            if ch != "\n":
                out[i] = " "
        elif state == "string":
            if ch == "\\" and i + 1 < len(source):
                out[i] = out[i + 1] = " "
                i += 2
                continue
            if ch == '"':
                out[i] = " "
                state = "code"
            elif ch != "\n":
                out[i] = " "
        i += 1
    return "".join(out)


def matching_brace(code: str, opening: int) -> int | None:
    depth = 0
    for i in range(opening, len(code)):
        if code[i] == "{":
            depth += 1
        elif code[i] == "}":
            depth -= 1
            if depth == 0:
                return i
    return None


def split_tests(path: Path, source: str) -> list[TestCase]:
    masked = mask_non_code(source)
    lines = source.splitlines()
    masked_lines = masked.splitlines()
    offsets: list[int] = []
    offset = 0
    for line in source.splitlines(keepends=True):
        offsets.append(offset)
        offset += len(line)

    tests: list[TestCase] = []
    i = 0
    while i < len(lines):
        if not masked_lines[i].strip().startswith("#[test]"):
            i += 1
            continue
        attr_start = i
        j = i + 1
        while j < len(lines) and not re.match(r"\s*(?:pub\s+)?fn\s+", masked_lines[j]):
            j += 1
        if j >= len(lines):
            break
        name_match = re.match(r"\s*(?:pub\s+)?fn\s+([A-Za-z0-9_]+)", masked_lines[j])
        if not name_match:
            i += 1
            continue
        opening = masked.find("{", offsets[j])
        if opening < 0:
            i += 1
            continue
        closing = matching_brace(masked, opening)
        if closing is None:
            i += 1
            continue
        end_line = source.count("\n", 0, closing)
        context_start = max(0, attr_start - 20)
        tests.append(
            TestCase(
                path=path,
                name=name_match.group(1),
                line=j + 1,
                attributes="\n".join(lines[attr_start:j]),
                body=source[offsets[j] : closing + 1],
                context="\n".join(lines[context_start : end_line + 1]),
            )
        )
        i = end_line + 1
    return tests


def macro_calls(code: str, pattern: re.Pattern[str]) -> list[tuple[int, str]]:
    calls: list[tuple[int, str]] = []
    for match in pattern.finditer(code):
        opening = code.find("(", match.end())
        if opening < 0:
            continue
        depth = 0
        for i in range(opening, len(code)):
            if code[i] == "(":
                depth += 1
            elif code[i] == ")":
                depth -= 1
                if depth == 0:
                    calls.append((match.start(), code[match.start() : i + 1]))
                    break
    return calls


def split_top_level(text: str) -> list[str]:
    parts: list[str] = []
    start = 0
    paren = bracket = brace = angle = 0
    for i, ch in enumerate(text):
        if ch == "(":
            paren += 1
        elif ch == ")":
            paren -= 1
        elif ch == "[":
            bracket += 1
        elif ch == "]":
            bracket -= 1
        elif ch == "{":
            brace += 1
        elif ch == "}":
            brace -= 1
        elif ch == "<":
            angle += 1
        elif ch == ">" and angle:
            angle -= 1
        elif ch == "," and paren == bracket == brace == angle == 0:
            parts.append(text[start:i].strip())
            start = i + 1
    parts.append(text[start:].strip())
    return parts


def macro_arguments(call: str) -> list[str]:
    opening = call.find("(")
    return split_top_level(call[opening + 1 : -1]) if opening >= 0 else []


def normalize_expression(expression: str) -> str:
    value = re.sub(r"\s+", "", expression)
    value = re.sub(r"\.clone\(\)$", "", value)
    value = value.removeprefix("&")
    while value.startswith("(") and value.endswith(")"):
        value = value[1:-1]
    return value


def has_informative_assertion(assertions: list[tuple[int, str]]) -> bool:
    for _, call in assertions:
        args = macro_arguments(call)
        if call.startswith(("assert_eq!", "assert_ne!", "debug_assert_eq!", "debug_assert_ne!")):
            if len(args) >= 2 and normalize_expression(args[0]) != normalize_expression(args[1]):
                return True
            continue
        expression = args[0] if args else call
        if ERROR_PREDICATE.search(expression):
            return True
        if re.match(r"\s*!", expression) and re.search(r"\.(?:is_ok|is_some)\s*\(", expression):
            return True
        if WEAK_PREDICATE.search(expression):
            if re.search(r"==|!=|<=|>=|(?<!-)[<>]", expression):
                return True
            continue
        else:
            return True
    return False


def direct_self_comparison(assertions: list[tuple[int, str]]) -> bool:
    for _, call in assertions:
        args = macro_arguments(call)
        if call.startswith(("assert_eq!", "debug_assert_eq!")) and len(args) >= 2:
            if normalize_expression(args[0]) == normalize_expression(args[1]):
                return True
        if args and re.search(r"\b([A-Za-z_]\w*)\s*-\s*\1\b", args[0]):
            return True
    return False


def assertion_is_inside_if(code: str, position: int) -> bool:
    for match in re.finditer(r"\bif\b[^{}]*\{", code):
        opening = code.find("{", match.start())
        closing = matching_brace(code, opening)
        if closing is not None and opening < position < closing:
            return True
    return False


def has_asserting_helper(code: str, helpers: set[str]) -> bool:
    opening = code.find("{")
    executable = code[opening + 1 :] if opening >= 0 else code
    calls = set(re.findall(r"\b([A-Za-z_]\w*)\s*(?:::\s*<[^>]*>)?\s*\(", executable))
    return not calls.isdisjoint(helpers)


def asserting_helpers(sources: dict[Path, str]) -> set[str]:
    functions: dict[str, str] = {}
    for source in sources.values():
        masked = mask_non_code(source)
        for match in re.finditer(r"\bfn\s+([A-Za-z_]\w*)\s*(?:<[^{};]*>)?\s*\([^;{}]*\)[^{;]*\{", masked):
            opening = masked.find("{", match.start())
            closing = matching_brace(masked, opening)
            if closing is not None:
                functions[match.group(1)] = masked[opening : closing + 1]
    helpers = {
        name
        for name, body in functions.items()
        if ASSERT_NAME.search(body) or re.search(r"\bpanic!", body)
    }
    names = set(functions)
    callers: dict[str, set[str]] = defaultdict(set)
    for caller, body in functions.items():
        for callee in re.findall(r"\b([A-Za-z_]\w*)\s*(?:::\s*<[^>]*>)?\s*\(", body):
            if callee in names and callee != caller:
                callers[callee].add(caller)
    queue = deque(helpers)
    while queue:
        callee = queue.popleft()
        for caller in callers.get(callee, ()):
            if caller not in helpers:
                helpers.add(caller)
                queue.append(caller)
    return helpers


def has_duplicate_branch(code: str) -> bool:
    for match in re.finditer(r"(?:\.|::)conditional\s*\(", code):
        opening = code.find("(", match.start())
        closing = None
        depth = 0
        for i in range(opening, len(code)):
            if code[i] == "(":
                depth += 1
            elif code[i] == ")":
                depth -= 1
                if depth == 0:
                    closing = i
                    break
        if closing is None:
            continue
        args = split_top_level(code[opening + 1 : closing])
        if len(args) >= 3 and normalize_expression(args[-2]) == normalize_expression(args[-1]):
            return True
    return False


def fixture_flags(test: TestCase, code: str) -> set[str]:
    flags: set[str] = set()
    if re.search(r"(?:vec!\s*)?\[\s*[^\[\];,]+\s*;\s*[2-9]\d*\s*\]", code):
        flags.add("uniform-fixture")

    for match in re.finditer(r"(?:vec!\s*)?\[([^\[\]]+)\]", code):
        if ";" in match.group(1):
            continue
        values = re.findall(r"(?<![A-Za-z_])[-+]?\d+(?:\.\d+)?(?:[eE][-+]?\d+)?", match.group(1))
        if len(values) >= 4 and len(set(values)) > 1 and values == list(reversed(values)):
            flags.add("palindromic-fixture")
            break

    before_assert = code[: ASSERT_NAME.search(code).start()] if ASSERT_NAME.search(code) else code
    values = re.findall(
        r"(?<![A-Za-z_])[-+]?(?:\d+\.\d*|\d*\.\d+|\d+)(?:[eE][-+]?\d+)?(?:f32|f64)?",
        before_assert,
    )
    numeric = []
    for value in values:
        cleaned = re.sub(r"(?:f32|f64)$", "", value)
        try:
            numeric.append(float(cleaned))
        except ValueError:
            pass
    if (
        len(numeric) >= 3
        and all(value in {-1.0, 0.0, 1.0} for value in numeric)
        and not DEGENERATE_PURPOSE.search(test.name)
    ):
        flags.add("zero-one-fixture")
    return flags


def classify(test: TestCase, helpers: set[str]) -> set[str]:
    code = mask_non_code(test.body)
    assertions = macro_calls(code, ASSERT_NAME)
    panics = macro_calls(code, re.compile(r"\bpanic!"))
    delegates = has_asserting_helper(code, helpers)
    flags: set[str] = set()

    if not assertions and not panics and "should_panic" not in test.attributes and not delegates:
        flags.add("no-observation")
    if assertions and not has_informative_assertion(assertions):
        flags.add("answer-blind")
    if direct_self_comparison(assertions):
        flags.add("self-comparison")
    if assertions and all(assertion_is_inside_if(code, position) for position, _ in assertions) and not panics:
        flags.add("conditional-only")

    discarded = re.findall(r"\blet\s+(_[A-Za-z0-9_]*)\s*=\s*[^;]+;", code)
    if any(name == "_" or len(re.findall(rf"\b{re.escape(name)}\b", code)) == 1 for name in discarded):
        flags.add("discarded-result")
    if has_duplicate_branch(code):
        flags.add("duplicate-branch")

    flags.update(fixture_flags(test, code))

    varied = bool(
        re.search(
            r"\bfor\b|\bwhile\b|\.iter\s*\(|\b(?:cases|rows|inputs|samples|fixtures)\s*=\s*[&]?\[",
            code,
        )
    )
    if not varied and assertions:
        flags.add("single-case")

    context_has_provenance = bool(PROVENANCE.search(test.context))
    literal_oracle = False
    for _, call in assertions:
        args = macro_arguments(call)
        if len(args) >= 2 and re.search(r"(?<![A-Za-z_])[-+]?\d+\.\d+", args[1]):
            literal_oracle = True
        elif args and re.search(r"[-,]\s*[-+]?\d+\.\d+", args[0]):
            literal_oracle = True
    if literal_oracle and not varied and not context_has_provenance:
        flags.add("unproven-literal-oracle")

    derived = False
    for match in re.finditer(
        r"let\s+\w*(?:expected|want|reference|exp|predicted|analytic|manual)\w*\s*(?::[^=]+)?=\s*([^;]+);",
        code,
        re.IGNORECASE,
    ):
        if ARITHMETIC.search(match.group(1)):
            derived = True
            break
    if not derived:
        for _, call in assertions:
            args = macro_arguments(call)
            expression = args[0] if args else ""
            inline = re.search(r"-\s*\(([^)]*[+*/][^)]*)\)\s*\)?\s*\.abs\s*\(\s*\)", expression)
            if inline and ARITHMETIC.search(inline.group(1)):
                derived = True
                break
    if derived:
        flags.add("derived-oracle")
        if not context_has_provenance:
            flags.add("unproven-derived-oracle")

    if RANDOM_SOURCE.search(test.body) and assertions and not has_informative_assertion(assertions):
        flags.add("random-survival-only")
    return flags


def site(test: TestCase) -> str:
    return f"{test.path}:{test.line}:{test.name}"


def audit(root: Path) -> dict[str, object]:
    paths = sorted(root.rglob("*.rs"))
    sources = {path: path.read_text() for path in paths}
    helpers = asserting_helpers(sources)
    inventory: dict[str, list[str]] = defaultdict(list)
    per_file: dict[str, dict[str, int]] = defaultdict(lambda: defaultdict(int))
    test_count = 0
    file_count = 0

    for path, source in sources.items():
        tests = split_tests(path, source)
        if tests:
            file_count += 1
        for test in tests:
            test_count += 1
            for flag in sorted(classify(test, helpers)):
                inventory[flag].append(site(test))
                per_file[str(path)][flag] += 1

    return {
        "root": str(root),
        "tests": test_count,
        "files": file_count,
        "counts": {key: len(value) for key, value in sorted(inventory.items())},
        "inventory": dict(sorted(inventory.items())),
        "per_file": {path: dict(sorted(flags.items())) for path, flags in sorted(per_file.items())},
    }


def print_report(result: dict[str, object]) -> None:
    tests = int(result["tests"])
    print(f"scanned {tests} tests in {result['files']} files\n")
    counts = result["counts"]
    assert isinstance(counts, dict)
    for name, count in sorted(counts.items(), key=lambda item: (-int(item[1]), item[0])):
        print(f"  {name:25} {int(count):5}  ({100 * int(count) / tests:5.1f}%)")

    risk_classes = {
        "answer-blind",
        "conditional-only",
        "discarded-result",
        "duplicate-branch",
        "no-observation",
        "palindromic-fixture",
        "random-survival-only",
        "self-comparison",
        "uniform-fixture",
        "unproven-derived-oracle",
        "unproven-literal-oracle",
        "zero-one-fixture",
    }
    per_file = result["per_file"]
    assert isinstance(per_file, dict)
    ranked = []
    for path, flags in per_file.items():
        score = sum(int(count) for flag, count in flags.items() if flag in risk_classes)
        if score:
            ranked.append((score, path, flags))
    print("\n  highest candidate density:")
    for score, path, flags in sorted(ranked, reverse=True)[:20]:
        detail = ", ".join(f"{name}={count}" for name, count in flags.items() if name in risk_classes)
        print(f"    {score:4}  {path}  [{detail}]")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT, help="Rust tests directory")
    parser.add_argument("--json", type=Path, default=DEFAULT_JSON, help="inventory output path")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if not args.root.is_dir():
        raise SystemExit(f"test root does not exist: {args.root}")
    result = audit(args.root)
    args.json.write_text(json.dumps(result, indent=2) + "\n")
    print_report(result)
    print(f"\nJSON inventory: {args.json}")


if __name__ == "__main__":
    main()
