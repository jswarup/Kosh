import os
import re

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
            
    new_content = "\n".join(formatted_lines)
    # Ensure trailing newline if original had it
    if content.endswith("\n") and not new_content.endswith("\n"):
        new_content += "\n"

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
