#!/usr/bin/env python3
"""
Script to fix color type mismatches in the Rust code.
Converts u32 color values to Color32 using helper functions.
"""

import re

# Read the file
with open('src/main.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Fix patterns where state.text_style.main_text_color is used (it's u32, needs Color32)
content = re.sub(
    r'state\.text_style\.main_text_color(?![\w])',
    'u32_to_color32(state.text_style.main_text_color)',
    content
)

# Fix patterns where state.text_style.sub_text_color is used
content = re.sub(
    r'state\.text_style\.sub_text_color(?![\w])',
    'u32_to_color32(state.text_style.sub_text_color)',
    content
)

# Fix patterns where state.text_style.panel_text_color is used (already partially fixed)
# Only fix remaining ones
content = re.sub(
    r'(?<!u32_to_color32\()state\.text_style\.panel_text_color(?![\w])',
    'u32_to_color32(state.text_style.panel_text_color)',
    content
)

# Fix gradient_colors access
content = re.sub(
    r'state\.theme\.gradient_colors\[(\d+)\](?![\w])',
    r'u32_to_color32(state.theme.gradient_colors[\1])',
    content
)

# Fix solid_color access
content = re.sub(
    r'state\.theme\.solid_color(?![\w])',
    'u32_to_color32(state.theme.solid_color)',
    content
)

# Write back
with open('src/main.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("Fixed color type conversions")
