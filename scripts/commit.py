#!/usr/bin/env python3

import pathlib
import os
import subprocess
import argparse
import random

def run(args):
    print(">", " ".join(args))
    result = subprocess.run(args)
    if result.returncode:
        print("Error! Command", args[0], "returned non-zero exit code", result.returncode)
        exit(result.returncode)

parser = argparse.ArgumentParser(description='Checks code & creates a git commit')
parser.add_argument('message', help='Commit message', nargs='?', default=None)
args = parser.parse_args()

dir = pathlib.Path(__file__).parent.parent.resolve()
os.chdir(dir)

run(["cargo", "check"])

run(["cargo", "clippy"])

run(["cargo", "fmt"])

run(["cargo", "test"])

run(["cargo", "doc"])

if args.message:
    run(["git", "add", "-A"])
    run(["git", "status"])
    message = args.message + " " + random.choice(["♦", "♣", "♥", "♠"])
    print(f"Message: {message}"),
    if input("Commit & Push (y/n)? ").strip().lower() == "y":
        run(["git", "commit", "-m", message])
        run(["git", "push"])
else:
    print("No commit message provided.")
