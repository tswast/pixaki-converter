import re

with open("crates/pixel-studio-pro-v2-converter/src/lib.rs", "r") as f:
    content = f.read()

# Just revert everything and start clean! This is a mess
