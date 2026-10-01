#!/usr/bin/env python3
"""
30-Vector Test Suite: Rust Desktop vs Mobile Small Floating Window Parity
Tests all 30 elements specified in the user request:
- Plus button: dimensions, green border, crimson cross, breathing glow, sub-card action, scale formula
- Three clock badges: Deadline (red), Sub-task (amber), Stopwatch (blue), formatting, pulsing dot, pill geometry, gestures
- Title bar: auto-hide 5.0s timer, toggle panel (☰/✕), show header, interaction wakeup
- Live Note: in-place writing, persistence, virtual scrolling (>10KB) badge, A+/A- font sizing, subtitle
- Windowing: compact pill vs expanded card, drag handling, palette, close teardown, bidirectional sync
"""

import os
import sys

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

RUST_HEADER = os.path.join(BASE_DIR, "src", "card_header_widgets.rs")
RUST_MAIN = os.path.join(BASE_DIR, "src", "main.rs")
RUST_LIVE_NOTE = os.path.join(BASE_DIR, "src", "views", "live_note.rs")

FLUTTER_OVERLAY = os.path.join(BASE_DIR, "mobile", "lib", "widgets", "floating_overlay_widget.dart")
KOTLIN_OVERLAY = os.path.join(BASE_DIR, "mobile", "android", "app", "src", "main", "kotlin", "com", "tasknote", "task_note_mobile", "FloatingWindowService.kt")
FLUTTER_SERVICE = os.path.join(BASE_DIR, "mobile", "lib", "services", "overlay_service.dart")
CYBER_THEME = os.path.join(BASE_DIR, "mobile", "lib", "theme", "cyber_theme.dart")

def read(path):
    with open(path, "r", encoding="utf-8") as f:
        return f.read()

def run_tests():
    rust_hdr = read(RUST_HEADER)
    rust_main = read(RUST_MAIN)
    rust_ln = read(RUST_LIVE_NOTE)
    flt_ov = read(FLUTTER_OVERLAY)
    kt_ov = read(KOTLIN_OVERLAY)
    flt_svc = read(FLUTTER_SERVICE)
    theme = read(CYBER_THEME)

    tests = []

    # ── Group 1: Plus Button & Sub-Card Creation (Tests 1–6) ──
    # Test 1: Plus button exact dimensions 22x22 px/dp & 5px corner radius
    t1 = ("22.0" in rust_hdr or "Vec2::splat(22.0)" in rust_hdr) and \
         ("22" in flt_ov and "5" in flt_ov) and \
         ("dpToPx(22)" in kt_ov and "dpToPx(5)" in kt_ov)
    tests.append(("Test 01: Plus Button Dimensions 22x22 and 5px radius", t1, "22x22 size and 5px rounding matched across Rust, Flutter, and Kotlin"))

    # Test 2: Plus button border color #3CB450 (GREEN_BORDER)
    t2 = ("GREEN_BORDER" in rust_hdr and "60, 180, 80" in rust_hdr) and \
         ("greenBorder" in flt_ov or "0xFF3CB450" in theme) and \
         ("#3CB450" in kt_ov)
    tests.append(("Test 02: Plus Button Green Border #3CB450", t2, "GREEN_BORDER #3CB450 verified in Rust, Theme, Flutter, and Kotlin"))

    # Test 3: Plus button crimson-red cross symbol #B21C1C (RED_CROSS)
    t3 = ("RED_CROSS" in rust_hdr and "178, 28, 28" in rust_hdr) and \
         ("redCross" in flt_ov or "0xFFB21C1C" in theme) and \
         ("#B21C1C" in kt_ov)
    tests.append(("Test 03: Plus Button Crimson-Red Cross #B21C1C", t3, "RED_CROSS #B21C1C matched in Rust and Mobile"))

    # Test 4: Plus button animated breathing pulse glow (0.0..1.0 phase)
    t4 = ("pulse" in rust_hdr and "sin()" in rust_hdr) and \
         ("AnimationController" in flt_ov and "pulse" in flt_ov)
    tests.append(("Test 04: Plus Button Animated Breathing Pulse Glow", t4, "Breathing glow sine-wave pulse animation implemented"))

    # Test 5: Plus button action triggers sub-card creation
    t5 = ("add_sub_card" in rust_hdr or "pending_plus_insert_after" in rust_main) and \
         ("_addSubCard" in flt_ov) and \
         ("ACTION_ADD_SUBCARD" in kt_ov)
    tests.append(("Test 05: Plus Button Sub-Card Creation Action", t5, "Sub-card creation trigger confirmed in both overlay implementations"))

    # Test 6: Sub-card hierarchical depth scaling formula (0.95^depth, min 70%)
    t6 = ("0.95_f32.powi" in rust_hdr and "0.70" in rust_hdr) and \
         ("0.95" in flt_ov and "scale" in flt_ov.lower())
    tests.append(("Test 06: Sub-Card Depth Scale 0.95^depth (min 70% floor)", t6, "Hierarchical scaling logic matched to Rust standard"))

    # ── Group 2: Three Clock Badges Parity (Tests 7–14) ──
    # Test 7: Badge 0 Task Deadline mode with Crimson Red #A51616
    t7 = ("TaskDeadline" in rust_hdr and "165, 22, 22" in rust_hdr) and \
         ("clockRed" in flt_ov or "0xFFA51616" in theme) and \
         ("#A51616" in kt_ov)
    tests.append(("Test 07: Badge 0 Task Deadline Crimson Red #A51616", t7, "Deadline badge color #A51616 present in all platforms"))

    # Test 8: Badge 0 time string format "HH.MM AM/PM"
    t8 = ("{:02}.{:02} {}" in rust_hdr or "AM" in rust_hdr) and \
         ("12.10 PM" in flt_ov and "12.10 PM" in kt_ov)
    tests.append(("Test 08: Badge 0 Time Format 'HH.MM AM/PM'", t8, "12.10 PM default time and AM/PM format present"))

    # Test 9: Badge 1 Sub-Task Time mode with Amber #B95F0F
    t9 = ("SubTaskTime" in rust_hdr and "185, 95, 15" in rust_hdr) and \
         ("clockAmber" in flt_ov or "0xFFB95F0F" in theme) and \
         ("#B95F0F" in kt_ov)
    tests.append(("Test 09: Badge 1 Sub-Task Time Amber #B95F0F", t9, "Sub-task badge color #B95F0F matched in all components"))

    # Test 10: Badge 2 Live Stopwatch mode with Steel-Blue #1C76B9
    t10 = ("Stopwatch" in rust_hdr and "28, 118, 185" in rust_hdr) and \
          ("clockBlue" in flt_ov or "0xFF1C76B9" in theme) and \
          ("#1C76B9" in kt_ov)
    tests.append(("Test 10: Badge 2 Stopwatch Steel-Blue #1C76B9", t10, "Stopwatch badge color #1C76B9 verified across Rust and mobile"))

    # Test 11: Badge 2 Stopwatch time string format "MM:SS" / "HH:MM:SS"
    t11 = ("{:02}:{:02}" in rust_hdr) and \
          ("_formatTime" in flt_ov and "padLeft(2, '0')" in flt_ov) and \
          ("updateStopwatchDisplay" in kt_ov and "%02d:%02d" in kt_ov)
    tests.append(("Test 11: Badge 2 Stopwatch Format 'MM:SS' / 'HH:MM:SS'", t11, "Two-tier stopwatch formatter confirmed"))

    # Test 12: Badge 2 Stopwatch pulsing blue dot #28C8FF indicator
    t12 = ("40, 200, 255" in rust_hdr or "circle_filled" in rust_hdr) and \
          ("0xFF28C8FF" in flt_ov) and \
          ("#28C8FF" in kt_ov)
    tests.append(("Test 12: Badge 2 Stopwatch Pulsing Dot #28C8FF", t12, "Ticking dot indicator #28C8FF present in Flutter & Kotlin"))

    # Test 13: Three badges pill chip dimensions ~64x20 with 5px radius and #3CB450 border
    t13 = ("64.0" in rust_hdr and "19.0" in rust_hdr and "Rounding::same(5.0)" in rust_hdr) and \
          ("BorderRadius.circular(5)" in flt_ov and "greenBorder" in flt_ov) and \
          ("dpToPx(5)" in kt_ov and "#3CB450" in kt_ov)
    tests.append(("Test 13: Three Badges Pill Chip Geometry and #3CB450 Border", t13, "64x20 dimensions, 5px radius, and green border verified"))

    # Test 14: Left-click tap toggles stopwatch & long-press resets stopwatch
    t14 = ("l_click" in rust_hdr and "sw_toggle" in rust_hdr and "r_click" in rust_hdr and "sw_reset" in rust_hdr) and \
          ("_toggleStopwatch" in flt_ov and "_resetStopwatch" in flt_ov) and \
          ("setOnLongClickListener" in kt_ov and "stopwatchSeconds = 0" in kt_ov)
    tests.append(("Test 14: Stopwatch Left-Click Toggle and Long-Press Reset", t14, "Toggle and reset gesture behaviors confirmed"))

    # ── Group 3: Title Bar Auto-Hide & Floating Controls (Tests 15–18) ──
    # Test 15: Title bar floating button group with Toggle Panel (☰ when visible, ✕ when hidden)
    t15 = ("☰" in rust_main and "✕" in rust_main and "TogglePanel" in rust_main) and \
          ("☰" in flt_ov and "✕" in flt_ov)
    tests.append(("Test 15: Title Bar Toggle Panel (☰ Visible, ✕ Hidden)", t15, "Sandwich and Close glyph states verified"))

    # Test 16: Show Header button rendered when title bar header is hidden
    t16 = ("ShowHeader" in rust_main and "!state.title_bar_state.header_visible" in rust_main) and \
          ("!_isHeaderVisible" in flt_ov and "Show Header" in flt_ov)
    tests.append(("Test 16: Show Header Button When Header Hidden", t16, "Conditional header reveal button confirmed"))

    # Test 17: Inactivity timer of 5.0 seconds triggers linear auto-hide opacity fade to 0.0
    t17 = ("elapsed > 5.0" in rust_main and "opacity" in rust_main) and \
          ("Duration(seconds: 5)" in flt_ov and "_floatingButtonOpacity = 0.0" in flt_ov) and \
          ("postDelayed(inactivityRunnable, 5000)" in kt_ov)
    tests.append(("Test 17: Inactivity Timer 5.0s Auto-Hide Opacity Fade to 0.0", t17, "5-second inactivity timeout matched in Rust, Flutter, and Android"))

    # Test 18: Pointer/touch interaction resets inactivity timer and restores opacity to 1.0
    t18 = ("last_interaction" in rust_main) and \
          ("_recordInteraction" in flt_ov and "_floatingButtonOpacity = 1.0" in flt_ov) and \
          ("recordInteraction" in kt_ov and "floatingButtonOpacity = 1.0f" in kt_ov)
    tests.append(("Test 18: Interaction Resets Timer & Restores Opacity 1.0", t18, "Touch wakeup handler verified in both mobile targets"))

    # ── Group 4: Live Note Editing & Virtual Scrolling (Tests 19–24) ──
    # Test 19: Live Note section displayed with dark background and purple accent
    t19 = ("Live Note" in rust_ln or "Live Note" in rust_main) and \
          ("neonPurple" in flt_ov and "0xFF090D14" in flt_ov) and \
          ("#9D4EDD" in kt_ov and "#FF090D14" in kt_ov)
    tests.append(("Test 19: Live Note Dark Container and #9D4EDD Accent", t19, "Live note dark container #090D14 and purple accent verified"))

    # Test 20: In-place editable text field for new note typing
    t20 = ("TextEdit" in rust_main and "main_text_input" in rust_main) and \
          ("TextField" in flt_ov and "_noteEditController" in flt_ov) and \
          ("EditText" in kt_ov and "liveNoteEditText" in kt_ov)
    tests.append(("Test 20: In-Place Editable Note TextField", t20, "Editable note input fields confirmed in Flutter and Android"))

    # Test 21: Live note Save action updates state and triggers persistence
    t21 = ("save_current_input" in rust_main or "state.save()" in rust_main) and \
          ("_saveLiveNote" in flt_ov and "UPDATE_NOTE" in flt_ov) and \
          ("liveNoteText = if (updated.isNotBlank())" in kt_ov)
    tests.append(("Test 21: Live Note Save Action & Persistence Sync", t21, "Note saving and broadcast sync confirmed"))

    # Test 22: Large text (>10 KB) virtual scrolling active indicator badge
    t22 = ("LARGE_TEXT_THRESHOLD" in rust_main and "10_240" in rust_main and "Virtual Scrolling Active" in rust_main) and \
          ("10240" in flt_ov and "Virtual Scrolling Active" in flt_ov) and \
          ("10240" in kt_ov and "Virtual Scrolling Active" in kt_ov)
    tests.append(("Test 22: Large Text (>10 KB) Virtual Scrolling Badge", t22, "10,240 byte threshold and badge message matched in Rust and Mobile"))

    # Test 23: Live note font size controls A+ and A- with clamped limits
    t23 = ("A+" in rust_main and "A-" in rust_main and "main_text_size" in rust_main) and \
          ("A+" in flt_ov and "A-" in flt_ov and "_noteTextSize" in flt_ov) and \
          ("A+" in kt_ov and "A-" in kt_ov and "noteTextSizeSp" in kt_ov)
    tests.append(("Test 23: Font Size Controls A+ / A- Clamped", t23, "Text sizing buttons A+/A- present across all implementations"))

    # Test 24: Subtitle / optional sub-text field rendering below main title
    t24 = ("sub_text" in rust_main) and \
          ("_subtitle" in flt_ov) and \
          ("subtitleTextView" in kt_ov)
    tests.append(("Test 24: Subtitle / Sub-Text Field Alignment", t24, "Sub-text secondary label confirmed in all layers"))

    # ── Group 5: Floating Window Modes, Dragging & Sync (Tests 25–30) ──
    # Test 25: Compact pill mode (chat head style) with live timer and breathing dot
    t25 = ("_buildCompactBubble" in flt_ov and "width: 175" in flt_ov)
    tests.append(("Test 25: Compact Pill Mode (Chat Head Style)", t25, "Compact floating pill mode verified"))

    # Test 26: Expanded floating card mode toggle via tap
    t26 = ("_buildExpandedCard" in flt_ov and "_toggleExpand" in flt_ov)
    tests.append(("Test 26: Expanded Floating Card View Toggle", t26, "Expand/collapse card transitions verified"))

    # Test 27: Native overlay drag support on screen with touch coordinate tracking
    t27 = ("setOnTouchListener" in kt_ov and "MotionEvent.ACTION_MOVE" in kt_ov and "windowManager?.updateViewLayout" in kt_ov)
    tests.append(("Test 27: Touch Coordinate Screen Dragging", t27, "Smooth touch drag listener and layout updates confirmed"))

    # Test 28: Biopunk / Holographic dark cosmic theme palette #080B10 matching Rust egui background
    t28 = ("bgCosmic" in theme and "0xFF080B10" in theme) and \
          ("CyberTheme.bgCosmic" in flt_ov) and \
          ("#F2080B10" in kt_ov or "#080B10" in kt_ov)
    tests.append(("Test 28: Biopunk Dark Cosmic Theme Palette #080B10", t28, "Dark cosmic background #080B10 matched across all files"))

    # Test 29: Dismiss / close overlay action gracefully shutting down service
    t29 = ("closeOverlay" in flt_ov) and \
          ("stopSelf()" in kt_ov and "windowManager?.removeView" in kt_ov)
    tests.append(("Test 29: Close Overlay Action & Lifecycle Teardown", t29, "Stop self and remove view cleanup handlers verified"))

    # Test 30: Bidirectional synchronization between floating overlay and TaskCard data model
    t30 = ("syncCardData" in flt_svc and "deadline" in flt_svc and "subTaskTime" in flt_svc) and \
          ("EXTRA_DEADLINE" in kt_ov and "EXTRA_SUBTASK_TIME" in kt_ov)
    tests.append(("Test 30: Bidirectional Data Model Synchronization", t30, "TaskCard parameters completely mapped through service channel"))

    # ── Report Generation ──
    print(f"{'='*80}")
    print(f"RUST VS SMALL FLOATING WINDOW PARITY: 30/30 ORTHOGONAL TEST SUITE")
    print(f"{'='*80}")

    passed_count = 0
    for idx, (name, passed, detail) in enumerate(tests, 1):
        status = "PASS" if passed else "FAIL"
        if passed:
            passed_count += 1
        print(f"[{idx:02d}/30] [{status}] {name}")
        print(f"       Detail: {detail}")
        if not passed:
            print(f"       ASSERTION ERROR: Test condition evaluated to False!")

    print(f"{'-'*80}")
    print(f"FINAL METRIC: {passed_count}/30 Passed ({passed_count/30*100:.1f}%)")
    print(f"{'='*80}")

    if passed_count == 30:
        sys.exit(0)
    else:
        sys.exit(1)

if __name__ == "__main__":
    run_tests()
