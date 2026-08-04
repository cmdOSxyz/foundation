#!/usr/bin/env bash
# tools/ci/check-docs.sh
# Prevents the six documentation conflicts reconciled by RFC-0001 from recurring.
# Wire into CI, e.g. a GitHub Actions step: `bash tools/ci/check-docs.sh`.

set -euo pipefail
fail=0

# a) Duplicate document numbers (CC.S.NN or CC.NN) within one directory.
#    Uniqueness is per directory, not global: the parallel docs/05-architecture/<area>/
#    subtrees each number their own documents 05.01..05.07 by design, so a global
#    check reports every area as a collision. docs/archive/ is historical.
dups=""
while IFS= read -r dir; do
  d="$(find "$dir" -maxdepth 1 -type f -exec \
        grep -hoE '^# [0-9]{2}(\.[0-9])?\.[0-9]{2}' {} + 2>/dev/null \
        | sort | uniq -d || true)"
  if [ -n "$d" ]; then
    dups="${dups}  ${dir}: $(echo "$d" | tr '\n' ' ')
"
  fi
done < <(find docs -type d -not -path 'docs/archive*' | sort)
if [ -n "$dups" ]; then
  echo "ERROR: duplicate document numbers within a directory:"
  printf '%s' "$dups"
  fail=1
fi

# b) Deprecated aliases must not appear.
#    Three exclusions, all legitimate:
#      - docs/archive/ holds historical records; rule 8 keeps them as written.
#      - the canonical glossary and RFC-0001 are the documents that *declare* these
#        aliases deprecated, so they necessarily contain them.
if grep -rIl \
     --exclude-dir=archive \
     --exclude='04.00-canonical-glossary.md' \
     --exclude='00.01-documentation-reconciliation.md' \
     -e 'Admin Desktop' \
     -e 'Admin Control Center' \
     -e 'Admin Runtime' \
     -e 'Plugin Marketplace' \
     -e 'Capability Marketplace' \
     docs/ CLAUDE.md README.md ROADMAP.md 2>/dev/null; then
  echo "ERROR: deprecated alias found in the files listed above."
  fail=1
fi

# c) Security docs outside the canonical tree must declare a canonical source
for f in \
  SECURITY.md \
  docs/04-terminology/04.10-security-model.md \
  docs/10-appendix/10.04-security-reference.md
do
  if [ -f "$f" ] && ! grep -qE 'Canonical source|Scope:' "$f"; then
    echo "ERROR: missing canonical-source header: $f"
    fail=1
  fi
done

if [ "$fail" -eq 0 ]; then
  echo "check-docs: OK"
fi
exit "$fail"
