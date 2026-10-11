#!/usr/bin/env python3
"""
ui_audit_types.py — Type Rigidity & Anti-Looseness Engine for CoSave Frontend

Agent-facing diagnostic engine for auditing TypeScript and Svelte code against
ADR 0006 (Relational Indexing), ADR 0007 (Anti-Looseness & Explicit Branching),
and AGENTS.md engineering invariants.

Outputs dense, structured, actionable diagnostics with clickable project-root
relative file paths, line anchors, AST-level regex matches, contextual code windows,
and explicit remediation recipes enabling automated agent rummaging and zero-guesswork refactoring.
"""

import argparse
import json
import os
import re
import subprocess
import sys
from collections import defaultdict
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple


def get_project_root() -> Path:
    """Finds project repository root via git rev-parse or parent traversal."""
    try:
        res = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            capture_output=True,
            text=True,
            check=True,
        )
        root = res.stdout.strip()
        if root:
            return Path(root)
    except Exception:
        pass
    # Fallback to 4 parents up from script directory: .agents/skills/ui-audit/scripts/
    return Path(__file__).resolve().parents[4]


PROJECT_ROOT = get_project_root()

KNOWN_BRANDS = [
    "UserId",
    "FamilyId",
    "MemberId",
    "AccountId",
    "TypeId",
    "CategoryId",
    "SubcategoryId",
    "ColorId",
    "TransactionId",
    "CurrencyId",
    "MinorUnits",
]

BRANDS_PATTERN = "|".join(KNOWN_BRANDS)

SEVERITY_FATAL = "FATAL"
SEVERITY_DEFENSIVE = "DEFENSIVE"
SEVERITY_STRICTNESS = "STRICTNESS"
SEVERITY_INFO = "INFO"

PATTERNS: List[Tuple[str, str, str, re.Pattern, str, str]] = [
    (
        "BLIND_ANY_CAST",
        SEVERITY_FATAL,
        "Blind any cast (as any)",
        re.compile(r"\bas\s+any\b"),
        "Casting to 'any' is strictly banned crate-wide. It demolishes the TypeScript compiler boundary.",
        "Replace with 'unknown' and decode through a fallible runtime schema decoder (parseX) or @total-typescript/shoehorn.",
    ),
    (
        "ANY_ANNOTATION",
        SEVERITY_FATAL,
        "Any type annotation (: any)",
        re.compile(r":\s*any\b|<any>"),
        "'any' type is strictly banned. Unknown external boundary data must enter as unknown.",
        "Replace with 'unknown', a generic parameter with constraints, or an explicit discriminated union.",
    ),
    (
        "NON_NULL_ASSERTION",
        SEVERITY_FATAL,
        "Non-null assertion (!)",
        re.compile(r"(?<=[a-zA-Z0-9_\)\]])!(?!=)"),
        "Non-null assertion (!) bypasses strict null checking and causes runtime unhandled exceptions if the invariant fails.",
        "Unwrap with expectPresent(val, action, message) to assert presence with a unique SCREAMING action token.",
    ),
    (
        "BLIND_BRAND_CAST",
        SEVERITY_FATAL,
        "Blind brand assertion (as Brand)",
        re.compile(rf"\bas\s+({BRANDS_PATTERN})\b"),
        "Blind type assertion 'as Brand' violates nominal typing and circumvents domain validation rules.",
        "Construct instances via companion smart constructors: *Id(raw) or MinorUnits(raw).",
    ),
    (
        "LOOSE_EQUALITY",
        SEVERITY_FATAL,
        "Loose equality (== or !=)",
        re.compile(r"(?<![!=])(?:==|!=)(?!=)"),
        "Loose equality triggers unpredictable JavaScript type coercion rules.",
        "Replace with strict equality: '===' or '!=='.",
    ),
    (
        "LOOSE_BRAND_UNION",
        SEVERITY_FATAL,
        "Loose brand union (Brand | ...)",
        re.compile(
            rf":\s*(?:(?:{BRANDS_PATTERN})\s*\|\s*(?:number|string|null|undefined)|(?:number|string|null|undefined)\s*\|\s*(?:{BRANDS_PATTERN}))\b"
        ),
        "Domain IDs must not degrade into primitive or loose unions. They create structural equivalence hazards.",
        "Require strict Brand or smart constructor conversion. If genuinely optional, type strictly as 'Brand | null'.",
    ),
    (
        "LEGACY_CONSTRUCTOR",
        SEVERITY_FATAL,
        "Legacy to*Id or toMinorUnits call",
        re.compile(r"\bto(?!IntId\b|StringId\b)(?:MinorUnits|[A-Z][a-zA-Z]*Id)\s*\("),
        "Legacy converter functions (to*Id, toMinorUnits) are completely eradicated in favor of companion constructors.",
        "Replace call with companion constructor: *Id(raw) or MinorUnits(raw).",
    ),
    (
        "FALLBACK_NULLISH",
        SEVERITY_DEFENSIVE,
        "Nullish coalescing default (??)",
        re.compile(r"\?\?"),
        "Nullish coalescing papers over invariant domain dependencies by injecting synthetic or empty defaults.",
        "If invariant: assert with expectPresent. If genuinely optional: use explicit branching (val !== undefined ? val : fallback).",
    ),
    (
        "FALLBACK_LOGICAL_OR",
        SEVERITY_DEFENSIVE,
        "Logical OR fallback (||)",
        re.compile(r"(?<!\|)\|\|(?!\|)"),
        "Logical OR fallback liquidates empty strings or zero numbers into fallback defaults via falsy coercion.",
        "Use explicit ternary with explicit condition or non-nullable store getter.",
    ),
    (
        "OPTIONAL_CHAINING",
        SEVERITY_DEFENSIVE,
        "Optional chaining (?.)",
        re.compile(r"\?\."),
        "Optional chaining on schema-guaranteed invariant relationships hides broken backend or store sync.",
        "Assert the parent record's presence directly through non-nullable store getters before reading properties.",
    ),
    (
        "OPTIONAL_PROP_MODIFIER",
        SEVERITY_DEFENSIVE,
        "Optional property modifier (?:)",
        re.compile(r"\b[a-zA-Z0-9_]+\?\s*:"),
        "Optional property modifier (?:) creates accidental 'undefined' states. Always provide the box: make the property required and use 'T | null' if genuinely empty.",
        "Remove '?' and declare explicitly: prop: Type | null. For UI option bags, verify if field is truly optional.",
    ),
    (
        "UNDEFINED_TYPE_ANNOTATION",
        SEVERITY_DEFENSIVE,
        "Undefined type annotation (| undefined)",
        re.compile(r":\s*[^;\n]*?\bundefined\b"),
        "Explicit 'undefined' type annotations represent uninitialized or missing state. Use 'T | null' to represent intentional emptiness.",
        "Replace '| undefined' with '| null', or unwrap state using invariant assertions.",
    ),
    (
        "RELATIONAL_SCAN",
        SEVERITY_STRICTNESS,
        "Ad-hoc .find() scan",
        re.compile(r"\.find\s*\("),
        "ADR 0006 strictly bans O(N) array scans (.find) in component templates, rows, and store getters.",
        "Maintain reactive SvelteMap indices (#map = $derived(new SvelteMap(...))) and expose getEntity(id) getters.",
    ),
    (
        "FALSY_CHECK",
        SEVERITY_STRICTNESS,
        "Implicit truthy/falsy check (if (!x))",
        re.compile(r"\bif\s*\(\s*![a-zA-Z0-9_\.]+\s*\)"),
        "Implicit truthy/falsy coercion treats 0, false, and empty strings identically to null/undefined.",
        "Perform explicit comparisons: val === false, count === 0, str === '', or val === null || val === undefined.",
    ),
    (
        "EXPLICIT_NULL_CHECK",
        SEVERITY_INFO,
        "Explicit null/undefined comparison",
        re.compile(r"(?:===|!==)\s*(?:null|undefined)\b|\b(?:null|undefined)\s*(?:===|!==)"),
        "Explicit null/undefined check.",
        "Review if this dependency is invariant (should use expectPresent) or legitimately optional.",
    ),
]


@dataclass
class Finding:
    file_path: str
    abs_path: str
    line_number: int
    column_start: int
    column_end: int
    severity: str
    category: str
    category_title: str
    matched_text: str
    snippet: str
    context_lines: Dict[int, str]
    rationale: str
    remediation: str

    @property
    def clickable_terminal_link(self) -> str:
        """Terminal-clickable path format (e.g. frontend/src/lib/toast.ts:21:9)."""
        return f"{self.file_path}:{self.line_number}:{self.column_start}"

    @property
    def clickable_markdown_link(self) -> str:
        """GitHub-flavored markdown clickable link (e.g. [toast.ts:21](file:///path#L21))."""
        basename = Path(self.file_path).name
        return f"[{basename}:L{self.line_number}](file://{self.abs_path}#L{self.line_number})"


def scan_file(file_path: Path, context_radius: int = 2) -> List[Finding]:
    findings: List[Finding] = []
    abs_resolved = file_path.resolve()
    try:
        rel_path = str(abs_resolved.relative_to(PROJECT_ROOT))
    except ValueError:
        rel_path = str(abs_resolved)

    try:
        with open(file_path, "r", encoding="utf-8") as f:
            lines = [line.rstrip("\r\n") for line in f.readlines()]
    except Exception as e:
        print(f"Error reading {file_path}: {e}", file=sys.stderr)
        return []

    for idx, line in enumerate(lines):
        line_num = idx + 1
        stripped = line.strip()

        # Skip pure comments
        if stripped.startswith("//") or stripped.startswith("/*") or stripped.startswith("*"):
            continue

        for cat_id, severity, title, pattern, rationale, remediation in PATTERNS:
            # Special exemptions:
            if "utils.ts" in rel_path and ("any" in cat_id or "BLIND_ANY" in cat_id):
                continue
            if "types.ts" in rel_path and cat_id == "NON_NULL_ASSERTION" and "!=" not in stripped:
                continue

            for match in pattern.finditer(line):
                start_col = match.start() + 1
                end_col = match.end() + 1

                # Capture window of context lines
                win_start = max(0, idx - context_radius)
                win_end = min(len(lines), idx + context_radius + 1)
                context_dict = {
                    (i + 1): lines[i] for i in range(win_start, win_end)
                }

                findings.append(
                    Finding(
                        file_path=rel_path,
                        abs_path=str(abs_resolved),
                        line_number=line_num,
                        column_start=start_col,
                        column_end=end_col,
                        severity=severity,
                        category=cat_id,
                        category_title=title,
                        matched_text=match.group(0),
                        snippet=stripped,
                        context_lines=context_dict,
                        rationale=rationale,
                        remediation=remediation,
                    )
                )

    return findings


def main():
    parser = argparse.ArgumentParser(
        description="Type Rigidity & Anti-Looseness Engine for CoSave Frontend."
    )
    parser.add_argument(
        "path",
        nargs="?",
        default="frontend/src/lib",
        help="Target folder or file to scan (default: frontend/src/lib)",
    )
    parser.add_argument(
        "--top-level-only",
        "-t",
        action="store_true",
        help="Only scan files in the top level of the directory (no recursion into subdirectories)",
    )
    parser.add_argument(
        "--filter",
        "-f",
        default=None,
        help="Filter file paths by regex pattern (e.g. '.*\\.svelte$', 'store\\.svelte\\.ts$')",
    )
    parser.add_argument(
        "--exclude-tests",
        action="store_true",
        help="Exclude *.test.ts and *.spec.ts files from the audit",
    )
    parser.add_argument(
        "--summary-only",
        "-s",
        action="store_true",
        help="Print only the executive summary table without line itemization",
    )
    parser.add_argument(
        "--category",
        "-c",
        default=None,
        help="Filter findings by category ID (e.g. BLIND_BRAND_CAST, LOOSE_BRAND_UNION)",
    )
    parser.add_argument(
        "--severity",
        default=None,
        choices=["FATAL", "DEFENSIVE", "STRICTNESS", "INFO"],
        help="Filter findings by severity",
    )
    parser.add_argument(
        "--markdown",
        "-m",
        action="store_true",
        help="Output findings formatted as clickable GitHub markdown links",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output raw findings and summary in JSON format for automated agent rummaging",
    )
    parser.add_argument(
        "--context",
        type=int,
        default=2,
        help="Number of context lines before and after each snippet (default: 2)",
    )

    args = parser.parse_args()

    raw_target = Path(args.path)
    root_path = (PROJECT_ROOT / raw_target) if not raw_target.is_absolute() else raw_target
    if not root_path.exists():
        print(f"Error: Path '{args.path}' does not exist (resolved: {root_path}).", file=sys.stderr)
        sys.exit(1)

    target_files: List[Path] = []
    if root_path.is_file():
        target_files.append(root_path)
    else:
        if args.top_level_only:
            for entry in os.scandir(root_path):
                if entry.is_file() and (entry.name.endswith(".ts") or entry.name.endswith(".svelte")):
                    target_files.append(Path(entry.path))
        else:
            for ext in ("*.ts", "*.svelte"):
                target_files.extend(root_path.rglob(ext))

    if args.exclude_tests:
        target_files = [
            f for f in target_files if not (f.name.endswith(".test.ts") or f.name.endswith(".spec.ts"))
        ]

    if args.filter:
        regex = re.compile(args.filter)
        target_files = [f for f in target_files if regex.search(str(f))]

    target_files.sort()

    all_findings: List[Finding] = []
    for f in target_files:
        findings = scan_file(f, context_radius=args.context)
        if args.category:
            findings = [item for item in findings if item.category == args.category]
        if args.severity:
            findings = [item for item in findings if item.severity == args.severity]
        all_findings.extend(findings)

    # Machine-readable JSON output for automated agent pipelines
    if args.json:
        payload = {
            "target": str(root_path.relative_to(PROJECT_ROOT) if root_path.is_relative_to(PROJECT_ROOT) else root_path),
            "files_scanned": len(target_files),
            "files_with_findings": len({f.file_path for f in all_findings}),
            "total_findings": len(all_findings),
            "severity_summary": {
                sev: sum(1 for f in all_findings if f.severity == sev)
                for sev in ["FATAL", "DEFENSIVE", "STRICTNESS", "INFO"]
            },
            "findings": [asdict(f) for f in all_findings],
        }
        print(json.dumps(payload, indent=2))
        return

    # Markdown format mode
    if args.markdown:
        findings_by_file = defaultdict(list)
        for finding in all_findings:
            findings_by_file[finding.file_path].append(finding)

        print(f"### Detailed Findings for `{args.path}` ({len(all_findings)} issues)\n")
        for file_path, items in sorted(findings_by_file.items()):
            first_item = items[0]
            print(f"#### 📄 [`{file_path}`](file://{first_item.abs_path})\n")
            for item in items:
                print(
                    f"- {item.clickable_markdown_link}: `[{item.severity}]` **{item.category_title}**"
                )
                print(f"  - Matched: `{item.matched_text}`")
                print(f"  - Code: `{item.snippet}`")
                print(f"  - Remedy: {item.remediation}\n")
        return

    # Ultra-verbose, agent-digestible text stream
    if not args.summary_only:
        findings_by_file = defaultdict(list)
        for finding in all_findings:
            findings_by_file[finding.file_path].append(finding)

        if findings_by_file:
            print(f"\n{'='*90}")
            print(f" UI TYPE AUDIT — DETAILED DIAGNOSTIC STREAM")
            print(f" Target: {args.path} | Total Findings: {len(all_findings)}")
            print(f"{'='*90}\n")

            for file_path, items in sorted(findings_by_file.items()):
                first_item = items[0]
                print(f"📄 {file_path}")
                print(f"   file://{first_item.abs_path}")
                print()
                for item in items:
                    sev_indicator = f"[{item.severity}]"
                    print(
                        f"  --> {item.clickable_terminal_link} {sev_indicator} [{item.category}] {item.category_title}"
                    )
                    print(f"      Link:        file://{item.abs_path}:{item.line_number}:{item.column_start}")
                    print(f"      Matched:     `{item.matched_text}`")
                    print(f"      Rationale:   {item.rationale}")
                    print(f"      Remedy:      {item.remediation}")
                    print(f"      Context:")
                    for lnum, lcontent in sorted(item.context_lines.items()):
                        marker = ">>>" if lnum == item.line_number else "   "
                        print(f"        {marker} {lnum:<4} | {lcontent}")
                    print()
                print("-" * 90)
        else:
            print(f"\n✨ Zero issues found in {args.path}!")

    # Executive Summary Table
    print(f"\n{'='*90}")
    print(f" EXECUTIVE SUMMARY: {args.path}")
    print(f"{'='*90}")
    print(f"Files scanned:         {len(target_files)}")
    files_with_issues = len({f.file_path for f in all_findings})
    print(f"Files with findings:   {files_with_issues}")
    print(f"Total findings:        {len(all_findings)}")
    print("-" * 90)

    # Severity Breakdown
    sev_counts = defaultdict(int)
    for f in all_findings:
        sev_counts[f.severity] += 1

    print(f"{'Severity':<20} | {'Count':<10} | {'Description'}")
    print(f"{'-'*20}-+-{'-'*10}-+-{'-'*55}")
    sev_descs = {
        "FATAL": "Strict compiler violations: any, !, as Brand, loose equality",
        "DEFENSIVE": "Defensive liquidations: ??, ||, ?., optional brand props",
        "STRICTNESS": "Relational/branching anti-patterns: .find(), if (!x)",
        "INFO": "Explicit null/undefined comparisons for invariant review",
    }
    for sev in ["FATAL", "DEFENSIVE", "STRICTNESS", "INFO"]:
        if sev in sev_counts:
            print(f"{sev:<20} | {sev_counts[sev]:<10} | {sev_descs.get(sev, '')}")
    print("-" * 90)

    # Category Breakdown
    cat_counts = defaultdict(int)
    for f in all_findings:
        cat_counts[f.category_title] += 1

    print(f"{'Category':<45} | {'Count':<10}")
    print(f"{'-'*45}-+-{'-'*10}")
    for title, count in sorted(cat_counts.items(), key=lambda x: -x[1]):
        print(f"{title:<45} | {count:<10}")

    print("-" * 90)

    # Top offending files
    file_counts = defaultdict(int)
    for f in all_findings:
        file_counts[f.file_path] += 1

    if file_counts:
        print("Top offending files by finding count:")
        for file_path, count in sorted(file_counts.items(), key=lambda x: -x[1])[:10]:
            print(f"  {count:<4} {file_path}")

    print(f"{'='*90}\n")


if __name__ == "__main__":
    main()
