"""
Homebrew bazel.
"""

def _homebrew_bottle_impl(ctx):
    # Python script to handle GHCR auth and download
    download_script = ctx.path("download_ghcr.py")
    ctx.file("download_ghcr.py", content = """
import urllib.request
import json
import os
import sys

url = sys.argv[1]
output = sys.argv[2]

# 1. Get Token
# Parse repository from URL or hardcode for homebrew/core/graphviz
# URL example: https://ghcr.io/v2/homebrew/core/graphviz/blobs/sha256:...
# Scope should be repository:homebrew/core/graphviz:pull
repo = "homebrew/core/graphviz"
token_url = f"https://ghcr.io/token?scope=repository:{repo}:pull&service=ghcr.io"

req = urllib.request.Request(token_url)
with urllib.request.urlopen(req) as resp:
    data = json.load(resp)
    token = data["token"]

# 2. Download Blob
req = urllib.request.Request(url)
req.add_header("Authorization", f"Bearer {token}")
# GHCR blobs are often redirects, urllib handles them but auth header needs to persist? 
# Standard urllib might drop auth on redirect?
# GHCR usually returns 307 to a signed URL. Unsigned redirect usually doesn't need auth.
# Let's try simple open.

with urllib.request.urlopen(req) as r, open(output, 'wb') as f:
    f.write(r.read())
""")

    output_file = "bottle.tar.gz"
    result = ctx.execute(["python3", download_script, ctx.attr.url, output_file])

    if result.return_code != 0:
        fail("Failed to download bottle: " + result.stderr + result.stdout)

    if ctx.attr.sha256:
        downloaded_sha = ctx.execute(["shasum", "-a", "256", output_file]).stdout.split(" ")[0]
        if downloaded_sha != ctx.attr.sha256:
            fail("SHA256 mismatch. Expected {}, got {}".format(ctx.attr.sha256, downloaded_sha))

    # Extract
    ctx.extract(output_file, stripPrefix = ctx.attr.strip_prefix)

    # Debug: List files
    # res = ctx.execute(["find", ".", "-maxdepth", "3"])
    # print("Extracted files:\n" + res.stdout)

    # Create BUILD file
    ctx.symlink(ctx.attr.build_file, "BUILD")

homebrew_bottle = repository_rule(
    implementation = _homebrew_bottle_impl,
    attrs = {
        "build_file": attr.label(mandatory = True),
        "sha256": attr.string(),
        "strip_prefix": attr.string(),
        "url": attr.string(mandatory = True),
    },
)
