import os
import re

def parse_block_content(text):
    parts = []
    buf = []
    brace_level = 0
    state = "CODE"
    i = 0
    while i < len(text):
        c = text[i]
        if state == "LINE_COMMENT":
            buf.append(c)
            if c == '\n':
                state = "CODE"
            i += 1
            continue
        elif state == "BLOCK_COMMENT":
            buf.append(c)
            if c == '*' and i + 1 < len(text) and text[i+1] == '/':
                buf.append('/')
                state = "CODE"
                i += 2
            else:
                i += 1
            continue
        elif state == "STRING":
            buf.append(c)
            if c == '"' and text[i-1] != '\\':
                state = "CODE"
            i += 1
            continue
        elif state == "CHAR":
            buf.append(c)
            if c == "'" and text[i-1] != '\\':
                state = "CODE"
            i += 1
            continue
            
        if c == '/' and i + 1 < len(text) and text[i+1] == '/':
            state = "LINE_COMMENT"
            buf.append("//")
            i += 2
            continue
        elif c == '/' and i + 1 < len(text) and text[i+1] == '*':
            state = "BLOCK_COMMENT"
            buf.append("/*")
            i += 2
            continue
        elif c == '"':
            state = "STRING"
            buf.append(c)
            i += 1
            continue
        elif c == "'":
            state = "CHAR"
            buf.append(c)
            i += 1
            continue
            
        if c == '{':
            brace_level += 1
        elif c == '}':
            brace_level -= 1
        elif c == ',' and brace_level == 0:
            parts.append("".join(buf))
            buf = []
            i += 1
            continue
            
        buf.append(c)
        i += 1
        
    if buf:
        parts.append("".join(buf))
        
    return [p for p in parts if p.strip()]

def format_use_part(part_text, depth, indent):
    part_text = part_text.strip()
    if not part_text:
        return ""
        
    brace_idx = -1
    state = "CODE"
    k = 0
    while k < len(part_text):
        c = part_text[k]
        if state == "LINE_COMMENT":
            if c == '\n':
                state = "CODE"
        elif state == "BLOCK_COMMENT":
            if c == '*' and k + 1 < len(part_text) and part_text[k+1] == '/':
                state = "CODE"
                k += 1
        elif state == "STRING":
            if c == '"' and part_text[k-1] != '\\':
                state = "CODE"
        elif state == "CHAR":
            if c == "'" and part_text[k-1] != '\\':
                state = "CODE"
        else:
            if c == '/' and k + 1 < len(part_text) and part_text[k+1] == '/':
                state = "LINE_COMMENT"
                k += 1
            elif c == '/' and k + 1 < len(part_text) and part_text[k+1] == '*':
                state = "BLOCK_COMMENT"
                k += 1
            elif c == '"':
                state = "STRING"
            elif c == "'":
                state = "CHAR"
            elif c == '{':
                brace_idx = k
                break
        k += 1
        
    if brace_idx == -1:
        if depth == 1:
            if part_text.startswith("use"):
                rest = part_text[3:].lstrip()
                return "use\t" + rest
        return indent + part_text
        
    path = part_text[:brace_idx].strip()
    if depth == 1:
        if path.startswith("use"):
            rest = path[3:].lstrip()
            path = "use\t" + rest
            
    nested_text = part_text[brace_idx+1:].rstrip()
    if nested_text.endswith("}"):
        nested_text = nested_text[:-1].rstrip()
        
    has_subs = False
    state = "CODE"
    k = 0
    while k < len(nested_text):
        c = nested_text[k]
        if state == "LINE_COMMENT":
            if c == '\n':
                state = "CODE"
        elif state == "BLOCK_COMMENT":
            if c == '*' and k + 1 < len(nested_text) and nested_text[k+1] == '/':
                state = "CODE"
                k += 1
        elif state == "STRING":
            if c == '"' and nested_text[k-1] != '\\':
                state = "CODE"
        elif state == "CHAR":
            if c == "'" and nested_text[k-1] != '\\':
                state = "CODE"
        else:
            if c == '/' and k + 1 < len(nested_text) and nested_text[k+1] == '/':
                state = "LINE_COMMENT"
                k += 1
            elif c == '/' and k + 1 < len(nested_text) and nested_text[k+1] == '*':
                state = "BLOCK_COMMENT"
                k += 1
            elif c == '"':
                state = "STRING"
            elif c == "'":
                state = "CHAR"
            elif c == '{':
                has_subs = True
                break
        k += 1
        
    if not has_subs:
        siblings = parse_block_content(nested_text)
        cleaned_siblings = [s.strip() for s in siblings if s.strip()]
        body = ", ".join(cleaned_siblings)
        if body:
            body = " " + body + " "
            
        if depth == 1:
            header = path + "{"
        else:
            header = indent + path + "\n" + indent + "{"
            
        return header + body + "}"
    else:
        siblings = parse_block_content(nested_text)
        next_indent = indent + "    "
        child_lines = []
        for sib in siblings:
            formatted_sib = format_use_part(sib, depth + 1, next_indent)
            if formatted_sib.strip():
                child_lines.append(formatted_sib)
                
        body_parts = []
        for line in child_lines:
            stripped = line.rstrip()
            if not stripped.endswith(","):
                line = stripped + ","
            body_parts.append(line)
            
        body = "\n".join(body_parts)
        
        if depth == 1:
            header = path + "{"
        else:
            header = indent + path + "{"
            
        closing = "\n" + indent + "}"
        return header + "\n" + body + closing

def do_format_use(use_stmt):
    use_stmt = use_stmt.strip()
    has_semi = use_stmt.endswith(";")
    if has_semi:
        use_stmt = use_stmt[:-1].strip()
    formatted = format_use_part(use_stmt, 1, "")
    if has_semi:
        formatted = formatted.rstrip() + ";"
    return formatted

def format_use_statements(content):
    i = 0
    result = []
    state = "CODE"
    
    while i < len(content):
        char = content[i]
        
        if state == "LINE_COMMENT":
            if char == '\n':
                state = "CODE"
            result.append(char)
            i += 1
            continue
        elif state == "BLOCK_COMMENT":
            if char == '*' and i + 1 < len(content) and content[i+1] == '/':
                result.append("*/")
                state = "CODE"
                i += 2
            else:
                result.append(char)
                i += 1
            continue
        elif state == "STRING":
            result.append(char)
            i += 1
            if char == '"' and content[i-2] != '\\':
                state = "CODE"
            continue
        elif state == "CHAR":
            result.append(char)
            i += 1
            if char == "'" and content[i-2] != '\\':
                state = "CODE"
            continue
            
        if char == '/' and i + 1 < len(content) and content[i+1] == '/':
            state = "LINE_COMMENT"
            result.append("//")
            i += 2
            continue
        elif char == '/' and i + 1 < len(content) and content[i+1] == '*':
            state = "BLOCK_COMMENT"
            result.append("/*")
            i += 2
            continue
        elif char == '"':
            state = "STRING"
            result.append(char)
            i += 1
            continue
        elif char == "'":
            state = "CHAR"
            result.append(char)
            i += 1
            continue
            
        is_use = False
        if content[i:i+3] == "use":
            is_start_boundary = (i == 0 or not content[i-1].isalnum() and content[i-1] != '_')
            is_end_boundary = (i + 3 < len(content) and not content[i+3].isalnum() and content[i+3] != '_')
            if is_start_boundary and is_end_boundary:
                is_use = True
                
        if is_use:
            use_chars = []
            j = i
            brace_level = 0
            use_state = "CODE"
            
            while j < len(content):
                c = content[j]
                if use_state == "LINE_COMMENT":
                    use_chars.append(c)
                    j += 1
                    if c == '\n':
                        use_state = "CODE"
                    continue
                elif use_state == "BLOCK_COMMENT":
                    use_chars.append(c)
                    if c == '*' and j + 1 < len(content) and content[j+1] == '/':
                        use_chars.append('/')
                        j += 2
                        use_state = "CODE"
                    else:
                        j += 1
                    continue
                elif use_state == "STRING":
                    use_chars.append(c)
                    j += 1
                    if c == '"' and content[j-2] != '\\':
                        use_state = "CODE"
                    continue
                elif use_state == "CHAR":
                    use_chars.append(c)
                    j += 1
                    if c == "'" and content[j-2] != '\\':
                        use_state = "CODE"
                    continue
                    
                if c == '/' and j + 1 < len(content) and content[j+1] == '/':
                    use_state = "LINE_COMMENT"
                    use_chars.append("//")
                    j += 2
                    continue
                elif c == '/' and j + 1 < len(content) and content[j+1] == '*':
                    use_state = "BLOCK_COMMENT"
                    use_chars.append("/*")
                    j += 2
                    continue
                elif c == '"':
                    use_state = "STRING"
                    use_chars.append(c)
                    j += 1
                    continue
                elif c == "'":
                    use_state = "CHAR"
                    use_chars.append(c)
                    j += 1
                    continue
                
                use_chars.append(c)
                if c == '{':
                    brace_level += 1
                elif c == '}':
                    brace_level -= 1
                elif c == ';':
                    if brace_level == 0:
                        j += 1
                        break
                j += 1
                
            use_stmt = "".join(use_chars)
            formatted_use = do_format_use(use_stmt)
            result.append(formatted_use)
            i = j
        else:
            result.append(char)
            i += 1
            
    return "".join(result)


def chunk_line(line):
    chunks = []
    current = ""
    state = "CODE" # CODE, STRING, CHAR, COMMENT
    i = 0
    while i < len(line):
        char = line[i]
        if state == "CODE":
            if char == '"':
                if current:
                    chunks.append(("CODE", current))
                    current = ""
                state = "STRING"
                current += char
            elif char == "'":
                if current:
                    chunks.append(("CODE", current))
                    current = ""
                state = "CHAR"
                current += char
            elif char == '/' and i + 1 < len(line) and line[i+1] == '/':
                if current:
                    chunks.append(("CODE", current))
                    current = ""
                chunks.append(("COMMENT", line[i:]))
                break
            else:
                current += char
        elif state == "STRING":
            current += char
            if char == '"' and line[i-1] != '\\':
                chunks.append(("STRING", current))
                current = ""
                state = "CODE"
        elif state == "CHAR":
            current += char
            if char == "'" and line[i-1] != '\\':
                chunks.append(("STRING", current))
                current = ""
                state = "CODE"
        i += 1
    if current:
        chunks.append((state, current))
    return chunks

def format_braces(line):
    if not line.strip():
        return line
    chunks = chunk_line(line)
    has_brace = False
    brace_chunk_idx = -1
    brace_char_idx = -1
    for idx, (type, val) in enumerate(chunks):
        if type == "CODE" and "{" in val:
            remaining_code = "".join(c[1] for c in chunks[idx+1:] if c[0] == "CODE").strip()
            if not remaining_code:
                has_brace = True
                brace_chunk_idx = idx
                brace_char_idx = val.rfind("{")
                break
    if has_brace:
        before_chunks = chunks[:brace_chunk_idx]
        brace_chunk_val = chunks[brace_chunk_idx][1]
        before_brace_in_chunk = brace_chunk_val[:brace_char_idx]
        after_brace_in_chunk = brace_chunk_val[brace_char_idx+1:]
        after_chunks = chunks[brace_chunk_idx+1:]

        code_before = "".join(c[1] for c in before_chunks if c[0] == "CODE") + before_brace_in_chunk
        if code_before.strip():
            line_before = ""
            for t, v in before_chunks:
                line_before += v
            line_before += before_brace_in_chunk
            line_before = line_before.rstrip()

            indent = line[:len(line) - len(line.lstrip())]
            line_brace = indent + "{"

            line_after = after_brace_in_chunk
            for t, v in after_chunks:
                line_after += v
            line_after = line_after.strip()

            if line_after:
                return line_before + "\n" + line_brace + " " + line_after + "\n"
            else:
                return line_before + "\n" + line_brace + "\n"
    return line

def format_line_code(line):
    # 1. Handle let statement indentation first
    stripped = line.lstrip()
    if stripped.startswith("let ") or stripped.startswith("let\t") or stripped.startswith("let\n") or stripped == "let":
        indent_chars = line[:len(line) - len(stripped)]
        spaces = indent_chars.count(' ')
        tabs = indent_chars.count('\t')
        levels = tabs + (spaces // 4)
        line = ('\t' * levels) + stripped

    # 2. Chunk line to only format CODE parts
    chunks = chunk_line(line)
    formatted_line = ""
    for type, val in chunks:
        if type == "CODE":
            # Tab after fn keyword
            val = re.sub(r'\bfn\s+', 'fn\t', val)
            # Space after open parenthesis unless followed by space or close parenthesis
            val = re.sub(r'\((?![ \)])', '( ', val)
            # Space after open generic bracket
            val = re.sub(r'([a-zA-Z0-9_]+)<(?![ \>])', r'\1< ', val)
        formatted_line += val
    return formatted_line

def format_file(file_path):
    with open(file_path, "r", encoding="utf-8") as f:
        content = f.read()

    lines = content.splitlines()
    formatted_lines = []
    
    for line in lines:
        # Format braces first (may split line into two)
        braced = format_braces(line)
        sub_lines = braced.splitlines()
        for sub_line in sub_lines:
            formatted_lines.append(format_line_code(sub_line))

    # Pass 2: Ensure empty lines around separator lines
    final_lines = []
    i = 0
    while i < len(formatted_lines):
        line = formatted_lines[i]
        is_separator = bool(re.match(r'^\s*//[-]+\s*$', line))
        if is_separator:
            # Ensure preceding line is empty (if not already empty and not at start of file)
            if final_lines and final_lines[-1].strip():
                final_lines.append("")
            
            final_lines.append(line)
            
            # Ensure succeeding line is empty (if not already empty and not at end of file)
            if i + 1 < len(formatted_lines) and formatted_lines[i+1].strip():
                final_lines.append("")
        else:
            final_lines.append(line)
        i += 1
            
    new_content = "\n".join(final_lines)
    # Ensure trailing newline if original had it
    if content.endswith("\n") and not new_content.endswith("\n"):
        new_content += "\n"

    # Enforce use formatting as the final step
    new_content = format_use_statements(new_content)

    if content != new_content:
        with open(file_path, "w", encoding="utf-8") as f:
            f.write(new_content)
        print(f"Formatted: {file_path}")

def main():
    src_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "src"))
    for root, dirs, files in os.walk(src_dir):
        for file in files:
            if file.endswith(".rs"):
                format_file(os.path.join(root, file))

if __name__ == "__main__":
    main()
