#!/usr/bin/env python3
"""
Fix blank lines that were introduced by the translation scripts.
Removes duplicate blank lines (more than 1 consecutive blank line → 1).
Also fixes lines where a newline was inserted after a comment, splitting it.
"""

import os
import sys

def fix_file(filepath):
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
    except:
        return False

    original = content

    # Fix pattern: comment line followed by blank line followed by another comment line
    # that should be on the same block (e.g., "// Role:\n\n// details\n\n// more")
    # This happens when a multi-line comment block gets extra blank lines inserted

    lines = content.split('\n')
    new_lines = []
    i = 0
    while i < len(lines):
        line = lines[i]

        # Check for blank line between two comment lines that were originally one block
        if (i + 2 < len(lines) and
            line.strip() == '' and
            i > 0 and
            lines[i-1].lstrip().startswith('//') and
            lines[i+1].lstrip().startswith('//') and
            # Both comment lines at same indentation
            len(lines[i-1]) - len(lines[i-1].lstrip()) == len(lines[i+1]) - len(lines[i+1].lstrip())):
            # Skip this blank line - it was likely introduced by translation
            i += 1
            continue

        new_lines.append(line)
        i += 1

    content = '\n'.join(new_lines)

    # Also remove triple+ blank lines
    while '\n\n\n\n' in content:
        content = content.replace('\n\n\n\n', '\n\n\n')

    if content != original:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
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

    print(f"\nFixed {fixed} files with blank line issues.")

if __name__ == '__main__':
    main()
