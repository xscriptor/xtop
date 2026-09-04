#!/usr/bin/env bash
# Structural audit for the xtop kernel (ROADMAP #47).
#
# Failing thresholds (exit 1 when violated):
#   - cfg(target_os) outside platform/ trees        : must be 0
#   - files over 600 lines                           : must be 0
#   - TODO/FIXME/XXX/HACK markers                    : must be 0
#   - `pub use ...::*` wildcard re-exports           : <= 30 (down from 30+)
#   - LOC per top-level area                         : <= 2400
#
# Non-failing metrics are printed for tracking (LOC, module count).
set -u
cd "$(dirname "$0")/.." || exit 1
SRC=src

fail=0
total_loc=0

echo "== xtop structural audit =="

# --- cfg(target_os) only inside platform/ trees ---------------------------------
bad_cfg=$(grep -rn "cfg(target_os" "$SRC" | grep -v "/platform/" | wc -l)
echo "cfg(target_os) outside platform/ trees: $bad_cfg (must be 0)"
[ "$bad_cfg" -ne 0 ] && fail=1

# --- oversized files --------------------------------------------------------------
echo "-- files > 300 lines (watch) and > 600 (fail):"
big600=$(find "$SRC" -name '*.rs' -exec wc -l {} + | awk '$1 > 600 && $2 != "total" {print $2": "$1" lines"}')
big300=$(find "$SRC" -name '*.rs' -exec wc -l {} + | awk '$1 > 300 && $2 != "total" {print $2": "$1" lines"}')
if [ -n "$big600" ]; then
    echo "$big600"
    echo "  ^ files over 600 lines"
    fail=1
fi
[ -n "$big300" ] && echo "$big300" || echo "  none > 300" 

# --- TODO/FIXME markers -------------------------------------------------------------
todos=$(grep -rn "TODO\|FIXME\|XXX\|HACK" "$SRC" | wc -l)
echo "TODO/FIXME/XXX/HACK markers: $todos (must be 0)"
[ "$todos" -ne 0 ] && fail=1

# --- wildcard re-exports -------------------------------------------------------------
wild=$(grep -rn "pub use .*::\*" "$SRC" | wc -l)
echo "wildcard 'pub use ...::*': $wild (allow <= 30)"
[ "$wild" -gt 30 ] && fail=1

# --- LOC per top-level area ------------------------------------------------------------
echo "-- LOC per area:"
for area in "$SRC"/*; do
    [ -d "$area" ] || continue
    loc=$(find "$area" -name '*.rs' -exec cat {} + | wc -l)
    total_loc=$((total_loc + loc))
    name=$(basename "$area")
    echo "  $name: $loc"
    [ "$loc" -gt 2400 ] && { echo "    ^ exceeds 2400"; fail=1; }
done
main_loc=$(wc -l < "$SRC/main.rs")
total_loc=$((total_loc + main_loc))
echo "main.rs: $main_loc / total kernel: $total_loc"

# --- module graph sanity (imports of kernel areas from lower layers) -------------------
echo "cross-area imports (info):"
grep -rn "use crate::" "$SRC" --include='*.rs' | awk -F'::' '{print $0}' | wc -l | xargs echo "  total use crate:: lines:"

if [ "$fail" -eq 0 ]; then
    echo "AUDIT OK"
else
    echo "AUDIT FAILED"
fi
exit $fail
