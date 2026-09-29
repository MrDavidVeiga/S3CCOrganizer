import os
import shutil
import sys
import zipfile

source = sys.argv[1]
dest = sys.argv[2]
os.makedirs(dest, exist_ok=True)

def is_package_bytes(path):
    try:
        with open(path, "rb") as f:
            return f.read(4) == b"DBPF"
    except OSError:
        return False

if zipfile.is_zipfile(source):
    with zipfile.ZipFile(source) as zf:
        members = [
            info for info in zf.infolist()
            if not info.is_dir() and info.filename.lower().endswith(".package")
        ]
        members.sort(key=lambda x: (x.file_size, x.filename.lower()))
        selected = members[:12]
        print(f"ZIP_PACKAGES_FOUND={len(members)}")
        print(f"ZIP_PACKAGES_SELECTED={len(selected)}")
        for index, info in enumerate(selected, 1):
            safe = os.path.basename(info.filename) or f"sample-{index}.package"
            out = os.path.join(dest, f"{index:02d}-{safe}")
            with zf.open(info) as src, open(out, "wb") as dst:
                shutil.copyfileobj(src, dst)
            print(f"EXTRACTED={info.filename}|{info.file_size}")
elif is_package_bytes(source):
    out = os.path.join(dest, "sample.package")
    shutil.copy2(source, out)
    print("DIRECT_PACKAGE=1")
else:
    print("UNSUPPORTED_SAMPLE=1")
    sys.exit(3)
