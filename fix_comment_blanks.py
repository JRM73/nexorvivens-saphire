#!/usr/bin/env python3
"""
Fix blank lines between a comment and its following code line.
When a comment is followed by a blank line and then code, remove the blank line.
Only do this when the original file wouldn't have had a blank line there.
"""

import os
import sys

def fix_file(filepath):
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            lines = f.readlines()
    except:
        return False

    original = list(lines)
    new_lines = []
    i = 0
    modified = False

    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        # Pattern: comment line → blank line → non-blank, non-comment line
        # This blank line was likely introduced by translation
        if (stripped.startswith('//') and
            i + 2 < len(lines) and
            lines[i + 1].strip() == '' and
            lines[i + 2].strip() != '' and
            not lines[i + 2].strip().startswith('//')):

            # Check if this is NOT a section divider (comment before a blank line before a function/struct)
            next_content = lines[i + 2].strip()

            # Keep blank line before top-level items (pub fn, fn, impl, struct, enum, etc.)
            # But remove it for inline comments before their associated code
            indent = len(line) - len(line.lstrip())
            if indent > 0:  # Indented comment = inline comment, remove blank
                new_lines.append(line)
                # Skip the blank line
                i += 2
                modified = True
                continue
            elif (next_content.startswith('pub ') or
                  next_content.startswith('fn ') or
                  next_content.startswith('impl ') or
                  next_content.startswith('struct ') or
                  next_content.startswith('enum ') or
                  next_content.startswith('#[') or
                  next_content.startswith('use ') or
                  next_content.startswith('mod ')):
                # Keep blank line before declarations (it's a doc comment pattern)
                new_lines.append(line)
                i += 1
                continue
            else:
                # Remove the blank line
                new_lines.append(line)
                i += 2  # Skip blank line
                modified = True
                continue

        new_lines.append(line)
        i += 1

    if modified:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.writelines(new_lines)
        return True
    return False

def main():
    base = "/mnt/Data1/code/saphire-lite/src"
    dirs = ['agent', 'algorithms', 'api', 'biology', 'body', 'care', 'cognition']
    root_files = [
        'main.rs', 'lib.rs', 'consciousness.rs', 'consensus.rs',
        'display.rs', 'emotions.rs', 'factory.rs', 'llm.rs',
        'metacognition.rs', 'neurochemistry.rs', 'pipeline.rs',
        'scenarios.rs', 'stimulus.rs', 'temperament.rs'
    ]

    all_files = []
    for f in root_files:
        path = os.path.join(base, f)
        if os.path.exists(path):
            all_files.append(path)
    for d in dirs:
        dir_path = os.path.join(base, d)
        for root, _, files in os.walk(dir_path):
            for f in files:
                if f.endswith('.rs'):
                    all_files.append(os.path.join(root, f))

    fixed = 0
    for filepath in sorted(all_files):
        if fix_file(filepath):
            fixed += 1
            rel = os.path.relpath(filepath, base)
            print(f"  FIXED: {rel}")

    print(f"\nFixed {fixed} files with comment→blank→code patterns.")

if __name__ == '__main__':
    main()
