# Plus Button Feature - Full Fix Prompt

## App Context

This is a **Pure Rust desktop GUI app** using:
- `winit` — window management, raw keyboard events
- `egui` — UI rendering, TextEdit, panels
- `wgpu` — GPU rendering backend

The app is a **card-based note/task system** on Windows. Cards are rendered in a scrollable list. There is a custom title bar with buttons, and a side panel with a main text input field that live-previews into the selected card.

---

## The Plus Button Feature — Exact Desired Behavior

### Rule 1: Plus button in title bar
- There is a `+` button in the title bar (NEON_LIME color, Font Awesome `\u{f067}`)
- Clicking it **always** creates a new blank card at index 0 (top of the list)
- After creation: auto-focus the input field AND the card's TextEdit
- Cursor should blink in both places immediately

### Rule 2: Keyboard `+` key — THE MOST IMPORTANT RULE
```
Plus key on keyboard = ALWAYS create a new card
NO EXCEPTIONS.
Even if the user is currently typing inside a TextEdit.
Even if a card is in edit mode.
Even if the input field has focus.
ALWAYS = new card.
```

### Rule 3: `Shift + Plus` = type "+" symbol
- When Shift is held down AND Plus is pressed → type the "+" character into whatever is focused
- This is the ONLY exception to Rule 2
- Do NOT intercept this — let egui handle it naturally

### Rule 4: New card behavior after creation
- Insert blank `Quote` at `quotes[0]` (top)
- Set `editing_quote_index = Some(0)`
- Set `current_quote_index = 0`
- Clear input fields (`main_text_input`, `sub_text_input`)
- Call `save()`
- Set `request_main_text_focus = true` — so input field gets cursor
- Card's TextEdit focus: trigger via `if state.editing_quote_index == idx_opt { text_edit_output.response.request_focus(); }`

---

## Current Implementation (What Already Exists)

### AppState fields relevant to this feature:
```rust
pub struct AppState {
    pub quotes: Vec<Quote>,
    pub editing_quote_index: Option<usize>,
    pub current_quote_index: usize,
    pub main_text_input: String,
    pub sub_text_input: String,
    pub request_main_text_focus: bool,
    pub shift_pressed: bool,
    pub show_plus_key_hint: bool,
    pub plus_key_hint_time: Option<Instant>,
    // ...
}
```

### Title bar action enum:
```rust
pub enum TitleBarAction {
    AddCardClicked,
    // ... other actions
}
```

### Existing Plus key handler in winit event loop (Location 1 — `src/main.rs` ~line 7062):
```rust
KeyCode::Equal => {
    if app_state.shift_pressed {
        // Let egui handle "+" input — do nothing here
    } else {
        // Add new card
        let new_quote = Quote::default(); // blank card
        app_state.quotes.insert(0, new_quote);
        app_state.current_quote_index = 0;
        app_state.editing_quote_index = Some(0);
        app_state.main_text_input.clear();
        app_state.sub_text_input.clear();
        app_state.request_main_text_focus = true;
        app_state.save();
    }
}
```

### Existing egui keyboard handler (Location 2 — `src/main.rs` ~line 4108, inside CentralPanel):
```rust
// Three approaches combined:
let plus_pressed = ui.input(|i| {
    let equals_pressed = i.key_pressed(egui::Key::Equals);
    let raw_equals = i.events.iter().any(|e| {
        if let egui::Event::Key { key, pressed, modifiers, .. } = e {
            *pressed && !modifiers.shift && matches!(key, egui::Key::Equals)
        } else {
            false
        }
    });
    let text_equals = i.events.iter().any(|e| {
        if let egui::Event::Text(text) = e {
            text == "="
        } else {
            false
        }
    });
    equals_pressed || raw_equals || text_equals
});

if plus_pressed {
    // Consume the event so "=" doesn't get typed
    ui.input_mut(|i| {
        i.events.retain(|e| {
            !matches!(e, egui::Event::Key { key: egui::Key::Equals, .. })
            && !matches!(e, egui::Event::Text(t) if t == "=")
        });
    });
    // Add new card logic...
}
```

### Card TextEdit auto-focus (Location 3 — `src/main.rs` ~line 3203, inside render_quote_card()):
```rust
// After rendering the card's TextEdit:
if state.editing_quote_index == idx_opt {
    text_edit_output.response.request_focus();
}
```

### Input field auto-focus (Location 4 — inside side panel TextEdit render):
```rust
let r = ui.add(egui::TextEdit::multiline(&mut state.main_text_input)...);
if state.request_main_text_focus {
    r.request_focus();
    state.request_main_text_focus = false;
}
```

---

## The Problem

The Plus key on keyboard is **NOT reliably creating a new card** when:
1. A TextEdit (card or input field) has focus — egui intercepts the keypress first
2. The winit handler at ~line 7062 never fires because egui consumed the event

**Root cause:** When egui has a focused TextEdit, it intercepts `Key::Equals` (the Plus key) and puts "=" into the text. The winit-level handler runs too late or not at all for that event.

The egui-level handler at ~line 4108 was added to fix this, but it has issues:
- `text_equals` check for `"="` fires on `Shift+Plus` too (because Shift+= is "+")
- The check order may allow events to slip through to TextEdit before consumption
- `key_pressed()` may not fire when a TextEdit has focus

---

## What Needs to Be Fixed

### Fix 1: Reliable Plus key detection that works even with focused TextEdit

The detection must happen **before** egui routes events to the TextEdit. This means:

```rust
// At the START of the egui update loop, BEFORE any panels are drawn:
let plus_without_shift = ctx.input(|i| {
    i.events.iter().any(|e| {
        matches!(e,
            egui::Event::Key {
                key: egui::Key::Equals,
                pressed: true,
                modifiers,
                ..
            } if !modifiers.shift
        )
    })
});

// Consume the event immediately to prevent it reaching TextEdit:
if plus_without_shift {
    ctx.input_mut(|i| {
        i.events.retain(|e| {
            !matches!(e,
                egui::Event::Key { key: egui::Key::Equals, pressed: true, modifiers, .. }
                if !modifiers.shift
            )
        });
    });
}
```

**Key difference:** Use `ctx.input()` not `ui.input()` — and call it at the **very top** of the frame before any panel/widget rendering. This way the event is consumed before any TextEdit can see it.

### Fix 2: Shift tracking must be reliable

The `shift_pressed` in AppState tracks shift via winit events. But for the egui-level check, use `modifiers.shift` from the egui event itself — don't rely on `app_state.shift_pressed` for the egui handler.

### Fix 3: Remove the `text_equals == "="` check

The `egui::Event::Text("=")` fires when `=` is typed without shift.
But `egui::Event::Text("+")` fires when `Shift+=` is pressed.
So checking for `Text("=")` is correct for detecting Plus-without-Shift.
However, if the event consumption doesn't happen before TextEdit processes it, this check is useless.

**Remove** the `text_equals` fallback entirely — it's unreliable. Only use `Event::Key` detection.

### Fix 4: Winit handler should also create card, not just egui handler

Keep both handlers active. The winit handler fires when the window has focus but no egui widget has keyboard focus. The egui handler fires when a TextEdit has focus. Together they cover all cases.

---

## Complete Correct Implementation

### Step 1: At the very top of your egui `update()` / frame function, BEFORE any `begin_frame` panel rendering:

```rust
// ── PLUS KEY GLOBAL HANDLER ──────────────────────────────────────────────
// Must run BEFORE any panels/widgets to intercept the event first.
let should_add_card_from_plus = ctx.input(|i| {
    i.events.iter().any(|e| {
        if let egui::Event::Key { key, pressed, modifiers, .. } = e {
            *pressed && !modifiers.shift && matches!(key, egui::Key::Equals)
        } else {
            false
        }
    })
});

if should_add_card_from_plus {
    // Consume event so "=" doesn't appear in any focused TextEdit
    ctx.input_mut(|i| {
        i.events.retain(|e| {
            if let egui::Event::Key { key, pressed, modifiers, .. } = e {
                // Keep the event ONLY if it's NOT our plus-without-shift
                !(*pressed && !modifiers.shift && matches!(key, egui::Key::Equals))
            } else {
                true // keep all other events
            }
        });
    });

    // Create new blank card at top
    let new_quote = Quote::default();
    app_state.quotes.insert(0, new_quote);
    app_state.current_quote_index = 0;
    app_state.editing_quote_index = Some(0);
    app_state.main_text_input.clear();
    app_state.sub_text_input.clear();
    app_state.request_main_text_focus = true;
    app_state.save();
}
// ─────────────────────────────────────────────────────────────────────────
```

### Step 2: Keep winit handler as-is (for when no egui widget has focus):

```rust
// In winit WindowEvent::KeyboardInput handler:
KeyCode::Equal => {
    if !app_state.shift_pressed {
        let new_quote = Quote::default();
        app_state.quotes.insert(0, new_quote);
        app_state.current_quote_index = 0;
        app_state.editing_quote_index = Some(0);
        app_state.main_text_input.clear();
        app_state.sub_text_input.clear();
        app_state.request_main_text_focus = true;
        app_state.save();
    }
    // If shift_pressed: do nothing, let egui add "+" to text
}
```

### Step 3: Remove the old egui handler at ~line 4108 inside CentralPanel

The old handler inside `CentralPanel` runs too late (after panel rendering starts). **Delete it entirely.** The new handler at the top of the frame replaces it.

### Step 4: Card TextEdit focus (keep as-is, this part works):

```rust
// In render_quote_card(), after TextEdit render:
if state.editing_quote_index == idx_opt {
    text_edit_output.response.request_focus();
}
```

### Step 5: Input field focus (keep as-is, this part works):

```rust
let r = ui.add(egui::TextEdit::multiline(&mut state.main_text_input)...);
if state.request_main_text_focus {
    r.request_focus();
    state.request_main_text_focus = false;
}
```

---

## Summary of Changes

| Location | Action |
|---|---|
| Top of frame (before panels) | ✅ ADD new `ctx.input()` plus key handler with event consumption |
| `CentralPanel` ~line 4108 | ❌ DELETE old egui plus key handler entirely |
| Winit `KeyCode::Equal` ~line 7062 | ✅ KEEP as-is (covers no-focus case) |
| `render_quote_card()` ~line 3203 | ✅ KEEP as-is |
| Input field focus ~line 4682 | ✅ KEEP as-is |

---

## Expected Behavior After Fix

| Scenario | Expected Result |
|---|---|
| App just opened, no card editing, press `+` | New card created ✅ |
| Currently editing a card, press `+` | New card created ✅ |
| Input field has focus, press `+` | New card created ✅ |
| Press `Shift + +` anywhere | "+" typed into focused field ✅ |
| Click `+` button in title bar | New card created ✅ |
| New card created | Cursor blinks in both input field and card ✅ |
