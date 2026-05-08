import pathlib
import subprocess

current_dir = pathlib.Path(__file__).parent
converter = current_dir / "target" / "debug" / "pixelartconvert.exe"

for filepath in current_dir.glob("**/*.pixaki"):
    print(filepath)
    subprocess.run([converter, str(filepath), str(filepath.parent / (filepath.stem + ".aseprite"))])
