#!/bin/bash
# =============================================================================
# gauntlet.sh — Interactive steganography test suite
# =============================================================================
# Usage:
#   bash gauntlet.sh              → interactive config wizard
#   bash gauntlet.sh --dry-run   → show planned tests without running them
# =============================================================================
set -euo pipefail

DRY_RUN=false
[[ "${1:-}" == "--dry-run" ]] && DRY_RUN=true

# ─── FIXED PATHS (not interactive) ───────────────────────────────────────────
BASE_URL="http://localhost:3000"
FILES_INTERNAL_URL="http://localhost:3004"
SUITE_DIR="/Users/yardendrori/Documents/steganography/documentation/Testing suite"
INPUTS_DIR="$SUITE_DIR/inputs"
CARRIERS_DIR="$INPUTS_DIR/carriers"
PAYLOADS_DIR="$INPUTS_DIR/payloads"
RUNS_DIR="$SUITE_DIR/runs"

# Default inputs — wizard can override
CARRIER_PATH="$CARRIERS_DIR/carrier-1600x960-60fps-240s.mp4"
PAYLOAD_PATH="$PAYLOADS_DIR/payload-text-512.txt"
PAYLOAD_SIZE=$(wc -c < "$PAYLOAD_PATH" | tr -d ' ')

# These are set per-run in run_wizard (after run folder name is known)
RUN_DIR=""
BASELINE_PATH=""   # per-run: $RUN_DIR/baseline.mp4
STEG_DIR=""        # per-run: $RUN_DIR/steg-objects/   (Group E videos kept here)
EXTRACTED_DIR=""   # per-run: $RUN_DIR/extracted/      (byte-comparison files, deleted after use)
RESULTS_CSV=""     # per-run: $RUN_DIR/results.csv

# ─── AVAILABLE COEFFICIENT PRESETS ───────────────────────────────────────────
# Extend this list to add your own.
# Grid reference: index = row*4 + col  (DC=0 is never targeted)
#   (0,1)=1  (1,0)=4  (1,1)=5  → near-DC, low-freq
#   (1,2)=6  (2,1)=9  (2,2)=10 → mid-freq
#   (2,3)=11 (3,2)=14           → high-freq  ← reference default
#   (3,3)=15                    → very high freq
PRESET_LABELS=("lowfreq"  "midfreq"   "highfreq" "veryhigh")
PRESET_INDICES=("1 4 5"   "6 9 10"    "11 14"    "15"      )

get_coeff_indices() {
    local label="$1"
    for i in "${!PRESET_LABELS[@]}"; do
        [[ "${PRESET_LABELS[$i]}" == "$label" ]] && { echo "${PRESET_INDICES[$i]}"; return; }
    done
    echo ""
}

# ─── INTERACTIVE CONFIG WIZARD ───────────────────────────────────────────────

ask() {
    # ask "Prompt text" "default value" → prints chosen value to stdout
    # Prompt itself goes to stderr so command substitution $(ask ...) only captures the answer
    local prompt="$1" default="$2" response
    if [[ -t 0 ]]; then
        printf "%s [%s]: " "$prompt" "$default" >&2
        read -r response
        echo "${response:-$default}"
    else
        echo "$default"
    fi
}

ask_yn() {
    # ask_yn "Prompt" "y|n" → prints y or n to stdout
    local prompt="$1" default="$2" response
    while true; do
        if [[ -t 0 ]]; then
            printf "%s (y/n) [%s]: " "$prompt" "$default" >&2
            read -r response
            response="${response:-$default}"
        else
            response="$default"
        fi
        case "$response" in y|Y) echo "y"; return ;; n|N) echo "n"; return ;; esac
        echo "  Please enter y or n." >&2
    done
}

run_wizard() {
    echo ""
    echo "══════════════════════════════════════════════════"
    echo "  Steganography Test Gauntlet — Configuration"
    echo "  Press Enter to accept each default [in brackets]"
    echo "══════════════════════════════════════════════════"

    # ── Auth ──────────────────────────────────────────────────────────────────
    echo ""
    echo "─── Authentication ───────────────────────────────"
    USERNAME=$(ask "Username" "your_username")
    PASSWORD=$(ask "Password" "your_password")

    # ── Input files ───────────────────────────────────────────────────────────
    echo ""
    echo "─── Input Files ──────────────────────────────────"
    echo "  Available carriers:  $(ls "$CARRIERS_DIR" 2>/dev/null | tr '\n' '  ')"
    CARRIER_PATH="$CARRIERS_DIR/$(ask "Carrier filename" "$(ls "$CARRIERS_DIR" | head -1)")"
    echo "  Available payloads:  $(ls "$PAYLOADS_DIR" 2>/dev/null | tr '\n' '  ')"
    PAYLOAD_PATH="$PAYLOADS_DIR/$(ask "Payload filename" "payload-text-512.txt")"
    PAYLOAD_SIZE=$(wc -c < "$PAYLOAD_PATH" | tr -d ' ')
    echo ""
    echo "  Both files need to exist on the server to embed."
    echo "  Skip upload if you already uploaded them in a previous run and have their IDs."
    UPLOAD_CARRIER=$(ask_yn "  Upload carrier now?" "y")
    [[ "$UPLOAD_CARRIER" == "n" ]] && CARRIER_ID=$(ask "    Server carrier file ID" "")
    UPLOAD_PAYLOAD=$(ask_yn "  Upload payload now?" "y")
    [[ "$UPLOAD_PAYLOAD" == "n" ]] && PAYLOAD_ID=$(ask "    Server payload file ID" "")

    # ── Test groups ───────────────────────────────────────────────────────────
    echo ""
    echo "─── Test Groups ──────────────────────────────────"
    echo "  Each group sweeps one variable while holding all others at the reference config."
    echo "  Cartesian mode ignores groups and runs every combination of all parameter arrays."
    echo ""
    RUN_GROUP_A=$(ask_yn "  A — Delta sweep: how much strength does each method need to survive?" "y")
    RUN_GROUP_B=$(ask_yn "  B — CPB sweep: more coefficients per bit = more robust but lower capacity?" "y")
    RUN_GROUP_C=$(ask_yn "  C — BPM sweep: larger blocks = safer alignment, smaller = more capacity?" "y")
    RUN_GROUP_D=$(ask_yn "  D — Coefficient band: do low/mid/high-freq coefficients behave differently?" "y")
    RUN_GROUP_E=$(ask_yn "  E — Multi-gen: do errors compound across re-encoding generations?" "y")
    RUN_CARTESIAN=$(ask_yn "  CART — Cartesian product of all arrays (can be large)?" "n")

    # ── Parameter arrays ──────────────────────────────────────────────────────
    echo ""
    echo "─── Parameter Arrays ─────────────────────────────"
    echo "  Space-separated values. Sweep groups use one array at a time;"
    echo "  cartesian mode crosses all of them simultaneously."
    echo ""
    read -ra DELTAS  <<< "$(ask "  Delta values — space-separated, no brackets (1-255)" "20 40 60 80 100 150 200 255")"
    read -ra CPBS    <<< "$(ask "  CPB values — space-separated, no brackets" "1 2 4 8 16")"
    read -ra BPMS    <<< "$(ask "  BPM values — space-separated, no brackets (1=4px 2=8px 4=16px)" "1 2 4")"
    read -ra METHODS <<< "$(ask "  Methods — space-separated, no brackets" "SS ISS STDM QIM")"
    echo "  Coeff presets available: ${PRESET_LABELS[*]}"
    echo "  (lowfreq=near-DC, midfreq=mid-band, highfreq=(2,3)+(3,2), veryhigh=(3,3))"
    read -ra COEFF_PRESET_NAMES <<< "$(ask "  Coeff presets — space-separated, no brackets" "lowfreq midfreq highfreq veryhigh")"

    # ── Reference config ──────────────────────────────────────────────────────
    echo ""
    echo "─── Reference Config ─────────────────────────────"
    echo "  Values held fixed when a sweep group varies its one variable."
    echo ""
    REF_DELTA=$(ask  "  Reference delta"  "150")
    REF_CPB=$(ask    "  Reference CPB"    "16")
    REF_BPM=$(ask    "  Reference BPM"    "4")
    REF_METHOD=$(ask "  Reference method" "SS")
    REF_COEFF_PRESET=$(ask "  Reference coeff preset" "highfreq")
    REF_SEED=$(ask   "  Seed (PN sequence for SS/ISS/STDM)" "sigma")

    # ── Multi-gen ─────────────────────────────────────────────────────────────
    echo ""
    echo "─── Group E — Multi-Gen ──────────────────────────"
    echo "  Each run embeds once then re-encodes at CRF 23 N times to simulate"
    echo "  the video being saved/shared/re-uploaded multiple times."
    echo "  Format for runs: method:delta pairs e.g.  SS:100 SS:255 ISS:255"
    echo ""
    MULTIGEN_GENS=$(ask "  Generations per run" "4")
    read -ra MULTIGEN_RUNS <<< "$(ask "  Runs" "SS:100 SS:255 ISS:255")"

    # ── Output ────────────────────────────────────────────────────────────────
    echo ""
    echo "─── Output ───────────────────────────────────────"
    echo "  Everything for this run goes into one folder:"
    echo "    baseline.mp4     — carrier re-encoded at CRF 23 with no embedding"
    echo "                       (quality reference: all PSNR/SSIM is measured against this)"
    echo "    results.csv      — one row per test: method, delta, cpb, bpm, PSNR, SSIM, diff bytes, PASS/FAIL"
    echo "    steg-objects/    — steg videos kept for inspection (Group E only; others are temp)"
    echo "    extracted/       — extracted payloads for byte comparison (auto-deleted after each test)"
    echo ""
    local ts
    ts=$(date +%Y%m%d_%H%M%S)
    local run_name
    run_name=$(ask "  Run folder name" "$ts")
    RUN_DIR="$RUNS_DIR/$run_name"
    BASELINE_PATH="$RUN_DIR/baseline.mp4"
    STEG_DIR="$RUN_DIR/steg-objects"
    EXTRACTED_DIR="$RUN_DIR/extracted"
    RESULTS_CSV="$RUN_DIR/results.csv"
    echo "  → $RUN_DIR"

    echo ""
    echo "══════════════════════════════════════════════════"
    echo "  Configuration complete."
    echo "══════════════════════════════════════════════════"
    echo ""
}

# ─── UTILITIES ────────────────────────────────────────────────────────────────

TOKEN=""
CARRIER_ID=""
PAYLOAD_ID=""

log()  { echo "[$(date +%H:%M:%S)] $*" >&2; }
die()  { log "FATAL: $*"; exit 1; }

make_coeffs() {
    local sel=($@) result="["
    for i in {0..15}; do
        [[ $i -gt 0 ]] && result+=","
        [[ " ${sel[*]} " =~ " $i " ]] && result+="true" || result+="false"
    done
    echo "${result}]"
}

reencode() { ffmpeg -y -i "$1" -c:v libx264 -crf 23 -movflags +faststart "$2" 2>/dev/null; }

measure_quality() {
    local ref="$1" tst="$2" tmp psnr ssim
    tmp=$(ffmpeg -i "$ref" -i "$tst" -lavfi "[0:v][1:v]psnr" -f null - 2>&1 || true)
    psnr=$(echo "$tmp" | grep -oE 'average:[0-9.inf]+' | tail -1 | grep -oE '[0-9.inf]+$' || echo "ERR")
    tmp=$(ffmpeg -i "$ref" -i "$tst" -lavfi "[0:v][1:v]ssim" -f null - 2>&1 || true)
    ssim=$(echo "$tmp" | grep -oE 'All:[0-9.]+' | tail -1 | grep -oE '[0-9.]+$' || echo "ERR")
    echo "$psnr $ssim"
}

diff_bytes() {
    [[ -f "$1" ]] || { echo "$PAYLOAD_SIZE"; return; }
    cmp -l <(head -c "$PAYLOAD_SIZE" "$1") <(head -c "$PAYLOAD_SIZE" "$2") \
        2>/dev/null | wc -l | tr -d ' ' || true
}

# ─── API ──────────────────────────────────────────────────────────────────────

do_login() {
    log "Logging in as $USERNAME..."
    local resp body
    if [[ "$USERNAME" == *@* ]]; then
        body="{\"email\":\"$USERNAME\",\"password\":\"$PASSWORD\",\"device_info\":\"gauntlet\"}"
    else
        body="{\"user_name\":\"$USERNAME\",\"password\":\"$PASSWORD\",\"device_info\":\"gauntlet\"}"
    fi
    resp=$(curl -s -X POST "$BASE_URL/api/auth/login" \
        -H "Content-Type: application/json" \
        -d "$body")
    TOKEN=$(echo "$resp" | jq -r '.access_token')
    [[ "$TOKEN" != "null" && -n "$TOKEN" ]] || die "Login failed: $resp"
}

upload_file() {
    local path="$1" name="$2"
    log "Uploading $name..."
    local init uid okey
    init=$(curl -s -X POST "$BASE_URL/api/files/initiate" -H "Authorization: Bearer $TOKEN")
    uid=$(echo "$init" | jq -r '.upload_id')
    okey=$(echo "$init" | jq -r '.object_key')

    local tmp parts_json="" part=1
    tmp=$(mktemp -d)
    split -b 10m "$path" "$tmp/chunk_"
    for chunk in "$tmp"/chunk_*; do
        local pr etag
        pr=$(curl -s -X POST \
            "$BASE_URL/api/files/upload-chunk?part_number=$part&upload_id=$uid&object_key=$okey" \
            -H "Authorization: Bearer $TOKEN" \
            -H "Content-Type: application/octet-stream" \
            --data-binary "@$chunk")
        etag=$(echo "$pr" | jq '.part.etag')
        [[ -n "$parts_json" ]] && parts_json+=","
        parts_json+="{\"part_number\":$part,\"etag\":$etag}"
        part=$((part + 1))
    done
    rm -rf "$tmp"

    local resp fid
    resp=$(curl -s -X POST "$BASE_URL/api/files/complete" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d "{\"upload_id\":\"$uid\",\"object_key\":\"$okey\",\"filename\":\"$name\",\"parts\":[$parts_json]}")
    fid=$(echo "$resp" | jq -r '.id')
    [[ "$fid" != "null" && -n "$fid" ]] || die "Upload failed for $name: $resp"
    log "  → ID $fid"
    echo "$fid"
}

download_file() { curl -sf -o "$2" -H "Authorization: Bearer $TOKEN" "$BASE_URL/api/files/$1/download"; }
delete_server_file() { curl -s -X DELETE -H "Authorization: Bearer $TOKEN" "$BASE_URL/api/files/$1" >/dev/null; }

mark_steg_object() {
    curl -sf -X PATCH "$FILES_INTERNAL_URL/internal/files/$1/embedded" 2>/dev/null \
        || log "Warning: mark_steg_object failed for $1"
}

do_embed() {
    local carrier_id="$1" payload_id="$2" method="$3" delta="$4" cpb="$5" bpm="$6" seed="$7" coeffs="$8"
    local body resp sid
    body=$(printf \
        '{"carrier_id":%s,"payload_id":%s,"configs":{"channels_to_embed":{"yuv":{"y":true,"cb":false,"cr":false}},"coefficients_to_embed":%s,"coefficients_per_bit":%s,"blocks_per_macroblock":%s,"delta":%s,"seed":"%s","method":"%s"}}' \
        "$carrier_id" "$payload_id" "$coeffs" "$cpb" "$bpm" "$delta" "$seed" "$method")
    resp=$(curl -s -X POST "$BASE_URL/api/embed/video" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d "$body" 2>&1)
    if echo "$resp" | grep -q "Payload too big"; then
        echo "INSUFFICIENT_CAPACITY"; return
    fi
    sid=$(echo "$resp" | jq -r '.id' 2>/dev/null)
    [[ "$sid" != "null" && -n "$sid" ]] && echo "$sid" || echo "FAIL"
}

do_extract() {
    local steg_id="$1" method="$2" delta="$3" cpb="$4" bpm="$5" seed="$6" coeffs="$7"
    local body resp eid
    body=$(printf \
        '{"steg_object_id":%s,"configs":{"channels_to_embed":{"yuv":{"y":true,"cb":false,"cr":false}},"coefficients_to_embed":%s,"coefficients_per_bit":%s,"blocks_per_macroblock":%s,"delta":%s,"seed":"%s","method":"%s"}}' \
        "$steg_id" "$coeffs" "$cpb" "$bpm" "$delta" "$seed" "$method")
    resp=$(curl -sf -X POST "$BASE_URL/api/extract/video" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d "$body" 2>&1) || { echo "FAIL"; return; }
    eid=$(echo "$resp" | jq -r '.id' 2>/dev/null)
    [[ "$eid" != "null" && -n "$eid" ]] && echo "$eid" || echo "FAIL"
}

# ─── CORE TEST RUNNER ─────────────────────────────────────────────────────────

# run_test <group> <method> <delta> <cpb> <bpm> <coeff_label> <coeff_indices> [<gen=1>] [<existing_steg_id>]
# If existing_steg_id is supplied, skip embedding and go straight to measure+extract.
run_test() {
    local group="$1" method="$2" delta="$3" cpb="$4" bpm="$5" clabel="$6" cidx="$7"
    local gen="${8:-1}" existing_steg_id="${9:-}"

    # QIM must use cpb=1 — override silently
    [[ "$method" == "QIM" && "$cpb" != "1" ]] && cpb="1"

    local tag="$group | $method | delta=$delta cpb=$cpb bpm=$bpm coeffs=$clabel gen=$gen"

    if [[ "$DRY_RUN" == "true" ]]; then
        echo "[DRY RUN] $tag"
        return
    fi

    do_login
    log "$tag"

    local coeffs_json
    coeffs_json=$(make_coeffs $cidx)

    # ── Embed (or reuse supplied steg ID) ──
    local steg_id
    if [[ -n "$existing_steg_id" ]]; then
        steg_id="$existing_steg_id"
    else
        steg_id=$(do_embed "$CARRIER_ID" "$PAYLOAD_ID" "$method" "$delta" "$cpb" "$bpm" "$REF_SEED" "$coeffs_json")
    fi

    if [[ "$steg_id" == "INSUFFICIENT_CAPACITY" ]]; then
        echo "$group,$method,$delta,$cpb,$bpm,$clabel,$gen,N/A,N/A,N/A,$PAYLOAD_SIZE,INSUFFICIENT_CAPACITY" >> "$RESULTS_CSV"
        log "  → INSUFFICIENT CAPACITY"
        return
    fi
    if [[ "$steg_id" == "FAIL" ]]; then
        echo "$group,$method,$delta,$cpb,$bpm,$clabel,$gen,ERR,ERR,$PAYLOAD_SIZE,$PAYLOAD_SIZE,EMBED_FAIL" >> "$RESULTS_CSV"
        log "  → EMBED FAILED"
        return
    fi

    # ── Download steg, measure quality ──
    local steg_path
    steg_path=$(mktemp /tmp/steg_XXXXXXXXXX).mp4
    download_file "$steg_id" "$steg_path"

    local quality psnr ssim
    quality=$(measure_quality "$BASELINE_PATH" "$steg_path")
    psnr=$(echo "$quality" | cut -d' ' -f1)
    ssim=$(echo "$quality" | cut -d' ' -f2)

    # ── Extract + compare ──
    local ext_id ext_path diff verdict
    ext_id=$(do_extract "$steg_id" "$method" "$delta" "$cpb" "$bpm" "$REF_SEED" "$coeffs_json")
    ext_path=$(mktemp /tmp/ext_XXXXXXXXXX)
    diff=$PAYLOAD_SIZE

    if [[ "$ext_id" != "FAIL" ]]; then
        download_file "$ext_id" "$ext_path"
        diff=$(diff_bytes "$ext_path" "$PAYLOAD_PATH")
        delete_server_file "$ext_id"
    fi
    [[ "$diff" -eq 0 ]] && verdict="PASS" || verdict="FAIL"

    echo "$group,$method,$delta,$cpb,$bpm,$clabel,$gen,$psnr,$ssim,$diff,$PAYLOAD_SIZE,$verdict" >> "$RESULTS_CSV"
    log "  → PSNR=$psnr SSIM=$ssim diff=$diff/$PAYLOAD_SIZE $verdict"

    delete_server_file "$steg_id"
    rm -f "$steg_path" "$ext_path"
}

# ─── MULTI-GEN RUNNER ─────────────────────────────────────────────────────────

run_multigen() {
    local method="$1" delta="$2" gens="$3"
    local cidx
    cidx=$(get_coeff_indices "$REF_COEFF_PRESET")
    local coeffs_json
    coeffs_json=$(make_coeffs $cidx)

    log "=== Multi-gen: $method delta=$delta ($gens gens) ==="

    if [[ "$DRY_RUN" == "true" ]]; then
        for g in $(seq 1 "$gens"); do
            echo "[DRY RUN] E | $method | delta=$delta cpb=$REF_CPB bpm=$REF_BPM coeffs=$REF_COEFF_PRESET gen=$g"
        done
        return
    fi

    local steg_id
    steg_id=$(do_embed "$CARRIER_ID" "$PAYLOAD_ID" "$method" "$delta" "$REF_CPB" "$REF_BPM" "$REF_SEED" "$coeffs_json")
    [[ "$steg_id" == "FAIL" ]] && { log "  Gen1 embed failed"; return; }

    local gen1_path="$STEG_DIR/multigen_${method}_d${delta}_gen1.mp4"
    download_file "$steg_id" "$gen1_path"

    # Gen1: extract + measure
    local q psnr ssim ext_id ext_path diff verdict
    q=$(measure_quality "$BASELINE_PATH" "$gen1_path")
    psnr=$(echo "$q" | cut -d' ' -f1); ssim=$(echo "$q" | cut -d' ' -f2)
    ext_id=$(do_extract "$steg_id" "$method" "$delta" "$REF_CPB" "$REF_BPM" "$REF_SEED" "$coeffs_json")
    ext_path=$(mktemp /tmp/ext_XXXXXXXXXX); diff=$PAYLOAD_SIZE
    if [[ "$ext_id" != "FAIL" ]]; then
        download_file "$ext_id" "$ext_path"; diff=$(diff_bytes "$ext_path" "$PAYLOAD_PATH")
        delete_server_file "$ext_id"
    fi
    [[ "$diff" -eq 0 ]] && verdict="PASS" || verdict="FAIL"
    delete_server_file "$steg_id"
    echo "E,$method,$delta,$REF_CPB,$REF_BPM,$REF_COEFF_PRESET,1,$psnr,$ssim,$diff,$PAYLOAD_SIZE,$verdict" >> "$RESULTS_CSV"
    log "  Gen1: PSNR=$psnr diff=$diff/$PAYLOAD_SIZE $verdict"
    rm -f "$ext_path"

    # Gen2..N
    local prev_path="$gen1_path"
    for g in $(seq 2 "$gens"); do
        local gen_path new_steg_id
        gen_path="$STEG_DIR/multigen_${method}_d${delta}_gen${g}.mp4"
        reencode "$prev_path" "$gen_path"
        new_steg_id=$(upload_file "$gen_path" "multigen_${method}_d${delta}_gen${g}.mp4")
        mark_steg_object "$new_steg_id"

        q=$(measure_quality "$BASELINE_PATH" "$gen_path")
        psnr=$(echo "$q" | cut -d' ' -f1); ssim=$(echo "$q" | cut -d' ' -f2)
        ext_id=$(do_extract "$new_steg_id" "$method" "$delta" "$REF_CPB" "$REF_BPM" "$REF_SEED" "$coeffs_json")
        ext_path=$(mktemp /tmp/ext_XXXXXXXXXX); diff=$PAYLOAD_SIZE
        if [[ "$ext_id" != "FAIL" ]]; then
            download_file "$ext_id" "$ext_path"; diff=$(diff_bytes "$ext_path" "$PAYLOAD_PATH")
            delete_server_file "$ext_id"
        fi
        [[ "$diff" -eq 0 ]] && verdict="PASS" || verdict="FAIL"
        delete_server_file "$new_steg_id"
        echo "E,$method,$delta,$REF_CPB,$REF_BPM,$REF_COEFF_PRESET,$g,$psnr,$ssim,$diff,$PAYLOAD_SIZE,$verdict" >> "$RESULTS_CSV"
        log "  Gen$g: PSNR=$psnr diff=$diff/$PAYLOAD_SIZE $verdict"
        rm -f "$ext_path"
        prev_path="$gen_path"
    done
}

# ─── TEST GROUP RUNNERS ───────────────────────────────────────────────────────

run_group_a() {
    log "=== GROUP A: Delta sweep ==="
    local ref_cidx ref_coeffs
    ref_cidx=$(get_coeff_indices "$REF_COEFF_PRESET")
    for method in "${METHODS[@]}"; do
        for delta in "${DELTAS[@]}"; do
            run_test "A" "$method" "$delta" "$REF_CPB" "$REF_BPM" "$REF_COEFF_PRESET" "$ref_cidx" || true
        done
    done
}

run_group_b() {
    log "=== GROUP B: CPB sweep ==="
    local ref_cidx
    ref_cidx=$(get_coeff_indices "$REF_COEFF_PRESET")
    for cpb in "${CPBS[@]}"; do
        run_test "B" "$REF_METHOD" "$REF_DELTA" "$cpb" "$REF_BPM" "$REF_COEFF_PRESET" "$ref_cidx" || true
    done
}

run_group_c() {
    log "=== GROUP C: BPM sweep ==="
    local ref_cidx
    ref_cidx=$(get_coeff_indices "$REF_COEFF_PRESET")
    for bpm in "${BPMS[@]}"; do
        run_test "C" "$REF_METHOD" "$REF_DELTA" "$REF_CPB" "$bpm" "$REF_COEFF_PRESET" "$ref_cidx" || true
    done
}

run_group_d() {
    log "=== GROUP D: Coefficient band sweep ==="
    for preset in "${COEFF_PRESET_NAMES[@]}"; do
        local cidx
        cidx=$(get_coeff_indices "$preset")
        [[ -z "$cidx" ]] && { log "  Unknown preset: $preset — skipping"; continue; }
        run_test "D" "$REF_METHOD" "$REF_DELTA" "$REF_CPB" "$REF_BPM" "$preset" "$cidx" || true
    done
}

run_group_e() {
    log "=== GROUP E: Multi-gen ==="
    for run in "${MULTIGEN_RUNS[@]}"; do
        local method delta
        method=$(echo "$run" | cut -d: -f1)
        delta=$(echo "$run" | cut -d: -f2)
        run_multigen "$method" "$delta" "$MULTIGEN_GENS"
    done
}

run_cartesian() {
    log "=== CARTESIAN: all combinations ==="
    local total=0
    for method in "${METHODS[@]}"; do
        for delta in "${DELTAS[@]}"; do
            for cpb in "${CPBS[@]}"; do
                for bpm in "${BPMS[@]}"; do
                    for preset in "${COEFF_PRESET_NAMES[@]}"; do
                        local cidx
                        cidx=$(get_coeff_indices "$preset")
                        [[ -z "$cidx" ]] && continue
                        run_test "CART" "$method" "$delta" "$cpb" "$bpm" "$preset" "$cidx" || true
                        total=$((total + 1))
                    done
                done
            done
        done
    done
    log "Cartesian: $total tests"
}

# ─── MAIN ─────────────────────────────────────────────────────────────────────

run_wizard

# Show test count estimate before starting
echo "─── Test Plan Preview ────────────────────────────"
SAVED_RESULTS_CSV="$RESULTS_CSV"
RESULTS_CSV="/dev/null"
DRY_RUN=true
[[ "$RUN_GROUP_A" == "y" ]]   && run_group_a
[[ "$RUN_GROUP_B" == "y" ]]   && run_group_b
[[ "$RUN_GROUP_C" == "y" ]]   && run_group_c
[[ "$RUN_GROUP_D" == "y" ]]   && run_group_d
[[ "$RUN_GROUP_E" == "y" ]]   && run_group_e
[[ "$RUN_CARTESIAN" == "y" ]] && run_cartesian
DRY_RUN=false
RESULTS_CSV="$SAVED_RESULTS_CSV"
echo ""

CONFIRM=$(ask_yn "Run all tests above?" "y")
[[ "$CONFIRM" == "n" ]] && { echo "Aborted."; exit 0; }

mkdir -p "$RUN_DIR" "$STEG_DIR" "$EXTRACTED_DIR" "$RUNS_DIR"
echo "group,method,delta,cpb,bpm,coeff_preset,gen,psnr,ssim,diff_bytes,total_bytes,result" > "$RESULTS_CSV"

do_login

log "Creating baseline (carrier re-encoded at CRF 23, no embedding)..."
reencode "$CARRIER_PATH" "$BASELINE_PATH"

[[ "$UPLOAD_CARRIER" == "y" ]] && CARRIER_ID=$(upload_file "$CARRIER_PATH" "carrier-1600x960-60fps-240s.mp4")
[[ "$UPLOAD_PAYLOAD" == "y" ]] && PAYLOAD_ID=$(upload_file "$PAYLOAD_PATH"  "payload-text-512.txt")

[[ "$RUN_GROUP_A" == "y" ]]   && run_group_a
[[ "$RUN_GROUP_B" == "y" ]]   && run_group_b
[[ "$RUN_GROUP_C" == "y" ]]   && run_group_c
[[ "$RUN_GROUP_D" == "y" ]]   && run_group_d
[[ "$RUN_GROUP_E" == "y" ]]   && run_group_e
[[ "$RUN_CARTESIAN" == "y" ]] && run_cartesian

echo ""
log "=== ALL DONE === Results: $RESULTS_CSV"
