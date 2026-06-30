import os
import re

def parse_use_str(s):
    # Remove 'use\t' and ';'
    s = s.strip().removeprefix("use").strip()
    if s.endswith(";"):
        s = s[:-1].strip()
    
    # Expand nested braces
    # e.g., crate::silo::{ Arr, Buff, IAccess }
    # This is a simple recursive descent parser
    def parse_tree(text):
        paths = []
        current = ""
        depth = 0
        brace_start = -1
        
        i = 0
        while i < len(text):
            if text[i] == '{':
                if depth == 0:
                    brace_start = i
                depth += 1
            elif text[i] == '}':
                depth -= 1
                if depth == 0:
                    inner = text[brace_start+1:i]
                    sub_paths = parse_tree(inner)
                    prefix = current.strip()
                    if prefix.endswith("::"):
                        prefix = prefix[:-2]
                    for sp in sub_paths:
                        paths.append(prefix + "::" + sp if prefix else sp)
                    current = ""
            elif text[i] == ',':
                if depth == 0:
                    if current.strip():
                        paths.append(current.strip())
                    current = ""
            else:
                if depth == 0:
                    current += text[i]
            i += 1
        if current.strip():
            paths.append(current.strip())
        return paths

    return [p.replace(" ", "") for p in parse_tree(s)]

class TrieNode:
    def __init__(self):
        self.children = {}
        self.is_leaf = False

def insert(node, path_parts):
    if not path_parts:
        node.is_leaf = True
        return
    head = path_parts[0]
    if head not in node.children:
        node.children[head] = TrieNode()
    insert(node.children[head], path_parts[1:])

def stringify(node, prefix="", indent=""):
    if node.is_leaf and not node.children:
        return prefix

    if len(node.children) == 1 and not node.is_leaf:
        k, v = list(node.children.items())[0]
        new_prefix = prefix + "::" + k if prefix else k
        return stringify(v, new_prefix, indent)

    # multiple children
    items = []
    for k, v in sorted(node.children.items()):
        items.append(stringify(v, k, indent + "    "))
    
    if not prefix:
        # root level, return list of groups
        return items
    
    # if it fits on one line (simple items)
    is_simple = all("\n" not in item for item in items)
    if is_simple and sum(len(i) for i in items) < 60:
        joined = ", ".join(items)
        return f"{prefix}::{{ {joined} }}"
    
    # multiline
    lines = [f"{prefix}::{{"]
    for i, item in enumerate(items):
        comma = "," if i < len(items) - 1 else ""
        lines.append(f"{indent}    {item}{comma}")
    lines.append(f"{indent}}}")
    return "\n".join(lines)


for root, _, files in os.walk("src"):
    for file in files:
        if not file.endswith(".rs"): continue
        path = os.path.join(root, file)
        with open(path, "r") as f:
            lines = f.readlines()
        
        use_lines = []
        other_lines = []
        in_use = False
        use_buffer = ""
        
        # We need to collect all `use` blocks at the top of the file
        # A `use` block might span multiple lines.
        top_matter = []
        i = 0
        while i < len(lines):
            line = lines[i]
            if line.startswith("//"):
                top_matter.append(line)
            elif line.startswith("use\t") or line.startswith("use "):
                # accumulate until ;
                stmt = line
                while not stmt.strip().endswith(";") and i + 1 < len(lines):
                    i += 1
                    stmt += lines[i]
                use_lines.append(stmt)
            elif line.strip() == "":
                top_matter.append(line)
            else:
                break
            i += 1
        
        if len(use_lines) <= 1:
            continue
            
        # parse all use statements
        all_paths = []
        for stmt in use_lines:
            all_paths.extend(parse_use_str(stmt))
            
        trie = TrieNode()
        for p in set(all_paths):
            insert(trie, p.split("::"))
            
        grouped_strs = stringify(trie)
        
        # reconstruct top matter
        new_lines = []
        
        # Find first non-comment non-empty line index in top_matter
        insert_idx = len(top_matter)
        for j, l in enumerate(top_matter):
            if l.startswith("//--") and j == 0:
                insert_idx = 1
                break
        
        final_file_lines = []
        if top_matter:
            final_file_lines.extend(top_matter[:insert_idx])
            
        for g in grouped_strs:
            final_file_lines.append(f"use\t{g};\n")
            
        if top_matter:
            final_file_lines.extend(top_matter[insert_idx:])
            
        final_file_lines.extend(lines[i:])
        
        with open(path, "w") as f:
            f.writelines(final_file_lines)

print("Done")
