import sys
import re

def format_file(filepath):
    with open(filepath, 'r') as f:
        lines = f.readlines()
    
    out = []
    for i, line in enumerate(lines):
        # 7. use statements
        line = re.sub(r'^use ', 'use\t', line)
        
        # 3. fn declarations
        line = re.sub(r'\bfn ', 'fn\t', line)
        
        # 4. let keyword
        line = re.sub(r'\blet (?! \t)', 'let  \t', line)
        
        # 1. braces on newline for struct, impl, fn
        # If line contains struct, impl or fn and ends with {
        if re.search(r'\b(struct|impl|fn\t)\b', line) and line.rstrip().endswith('{'):
            line = line.rstrip()[:-1].rstrip() + '\n{\n'
            
        # 8. return must be on separate line (for inline ones)
        # e.g., if curr != U8(b'"') { return None; }
        if re.search(r'\{ return [^}]+; \}', line):
            indent = re.match(r'^\s*', line).group(0)
            line = re.sub(r'\{ return ([^}]+); \}', r'{\n' + indent + r'    return \1;\n' + indent + r'}', line)

        # 2. Spacing in brackets
        # Open parenthesis MUST have trailing space if not empty
        line = re.sub(r'\((?!\s|\))', '( ', line)
        # Open angular bracket MUST have trailing space if not empty
        line = re.sub(r'<(?!\s|>|=)', '< ', line)
        
        out.append(line)
        
    with open(filepath, 'w') as f:
        f.writelines(out)

for arg in sys.argv[1:]:
    format_file(arg)
