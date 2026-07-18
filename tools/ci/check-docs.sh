#!/usr/bin/env bash
# tools/ci/check-docs.sh
# Prevents the six documentation conflicts reconciled by RFC-0001 from recurring.
# Wire into CI, e.g. a GitHub Actions step: `bash tools/ci/check-docs.sh`.

set -euo pipefail
fail=0

# a) Duplicate document numbers (CC.S.NN or CC.NN) across docs/
dups="$(grep -rhoE '^# [0-9]{2}(\.[0-9])?\.[0-9]{2}' docs/ 2>/dev/null \
  | sort | uniq -d || true)"
if [ -n "$dups" ]; then
  echo "ERROR: duplicate document numbers:"
  echo "$dups"
  fail=1
fi

# b) Deprecated aliases must not appear
if grep -rIl \
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
