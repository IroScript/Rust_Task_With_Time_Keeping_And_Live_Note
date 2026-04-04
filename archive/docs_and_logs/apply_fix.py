import sys

def process_file():
    with open("src/main.rs", "r", encoding="utf-8") as f:
        content = f.read()

    def find_block(text, start_str):
        start_idx = text.find(start_str)
        if start_idx == -1: return None, None, None
        
        brace_idx = text.find("{", start_idx)
        if brace_idx == -1: return None, None, None
        
        brace_count = 1
        idx = brace_idx + 1
        in_string = False
        in_comment = False
        
        while idx < len(text):
            if text[idx:idx+2] == "//" and not in_string:
                in_comment = True
                idx += 2
                continue
            if text[idx] == "\n" and in_comment:
                in_comment = False
                idx += 1
                continue
                
            if in_comment:
                idx += 1
                continue
                
            if text[idx] == '"' and text[idx-1] != '\\':
                in_string = not in_string
                idx += 1
                continue
                
            if not in_string:
                if text[idx] == '{':
                    brace_count += 1
                elif text[idx] == '}':
                    brace_count -= 1
                    if brace_count == 0:
                        return start_idx, brace_idx, idx
            idx += 1
        return None, None, None

    # --- Change 1: render_quote_card ---
    # Find "if !is_being_edited {"
    c1_start, c1_brace, c1_end = find_block(content, "if !is_being_edited {")
    if c1_start is not None:
        # Check if there is an `else {` immediately after
        else_search = content[c1_end+1:c1_end+20]
        if "else" in else_search:
            # Find the else block
            else_start_idx = content.find("else", c1_end)
            c1_else_start, c1_else_brace, c1_else_end = find_block(content[else_start_idx:], "{")
            if c1_else_start is not None:
                # Actual indices in main content
                actual_else_brace = else_start_idx + c1_else_brace
                actual_else_end = else_start_idx + c1_else_end
                
                # Extract branch B contents
                branch_b_content = content[actual_else_brace+1:actual_else_end]
                
                # Replace everything from `if !is_being_edited` to the end of the `else` block
                replacement = "\n            ui.set_min_height(120.0);\n" + branch_b_content
                
                content = content[:c1_start] + replacement + content[actual_else_end+1:]
                print("Change 1 applied successfully.")
            else:
                print("Failed to find else block for Change 1")
        else:
            print("Failed to find else keyword for Change 1")
    else:
        print("Failed to find Change 1 block")

    # --- Change 2: render_main_content ---
    # Replace the duplicate `if has_editing` block
    # We want to keep branch A (or B, they are identical) and remove the if/else entirely.
    c2_start, c2_brace, c2_end = find_block(content, "if has_editing {")
    if c2_start is not None:
        else_search = content[c2_end+1:c2_end+20]
        if "else" in else_search:
            else_start_idx = content.find("else", c2_end)
            c2_else_start, c2_else_brace, c2_else_end = find_block(content[else_start_idx:], "{")
            if c2_else_start is not None:
                actual_else_end = else_start_idx + c2_else_end
                
                # Extract branch A contents
                branch_a_content = content[c2_brace+1:c2_end]
                
                # Replace the entire if/else block with branch A content
                content = content[:c2_start] + branch_a_content + content[actual_else_end+1:]
                print("Change 2 applied successfully.")
            else:
                print("Failed to find else block for Change 2")
        else:
            print("Failed to find else keyword for Change 2")
    else:
        print("Failed to find Change 2 block")

    with open("src/main.rs", "w", encoding="utf-8") as f:
        f.write(content)

if __name__ == "__main__":
    process_file()
