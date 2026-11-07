#!/usr/bin/env python3
"""Revisión de archivos del índice Git sin imprimir posibles secretos."""
import pathlib
import re
import subprocess
import sys

root = pathlib.Path(__file__).resolve().parents[1]
paths = subprocess.check_output(
    ["git", "ls-files", "-z"], cwd=root
).decode().split("\0")
secret_patterns = [
    re.compile(rb"gh[pousr]_[A-Za-z0-9]{30,}"),
    re.compile(rb"github_pat_[A-Za-z0-9_]{30,}"),
    re.compile(rb"AKIA[A-Z0-9]{16}"),
    re.compile(rb"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----"),
    re.compile(rb"https?://[^\s/:]+:[^\s/@]+@"),
    re.compile(rb"/(?:home|Users)/[A-Za-z0-9_.-]+/"),
]
excluded_parts = {"target", ".tools", ".local", ".artifacts", ".gradle", ".aws", "jniLibs"}
excluded_suffixes = {".apk", ".aab", ".jks", ".keystore", ".pem", ".key"}
issues = []
for name in filter(None, paths):
    path = pathlib.PurePosixPath(name)
    if (
        excluded_parts.intersection(path.parts)
        or path.suffix in excluded_suffixes
        or path.name == "local.properties"
        or (path.name.startswith(".env") and path.name != ".env.example")
    ):
        issues.append(f"Archivo local o privado en el índice: {name}")
        continue
    # Inspeccionar lo que se publicará, incluso si la copia de trabajo es distinta.
    data = subprocess.check_output(["git", "show", f":{name}"], cwd=root)
    if any(pattern.search(data) for pattern in secret_patterns):
        issues.append(f"Revisar posible secreto o ruta personal: {name}")
if issues:
    print("\n".join(issues), file=sys.stderr)
    sys.exit(1)
print(f"Revisión del índice: {len(list(filter(None, paths)))} archivos; sin patrones detectados.")
print("Complementar con revisión del diff; esta comprobación no detecta todos los secretos.")
