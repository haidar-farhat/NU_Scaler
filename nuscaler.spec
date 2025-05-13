# -*- mode: python ; coding: utf-8 -*-
from PyInstaller.utils.hooks import collect_submodules, collect_data_files

block_cipher = None

# Get all hidden imports for required modules
pyside6_imports = collect_submodules('PySide6')
win32_imports = collect_submodules('win32')
core_imports = ['nu_scaler_py', 'nu_scaler_core']

# Collect all hidden imports
hidden_imports = pyside6_imports + win32_imports + core_imports

# Data files
data_files = collect_data_files('nu_scaler_core')
data_files += collect_data_files('nu_scaler_py')

a = Analysis(
    ['app_launcher.py'],
    pathex=[],
    binaries=[],
    datas=data_files,
    hiddenimports=hidden_imports,
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=['matplotlib', 'scipy', 'pandas', 'tkinter', 'PyQt5', 'PyQt6', 'wx'],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

# Exclude unnecessary PySide6 modules to reduce size
def exclude_from_analysis(analysis, excludes):
    """Exclude modules from the given analysis."""
    for exclude in excludes:
        found = [x for x in analysis.binaries if exclude in x[0]]
        for f in found:
            analysis.binaries.remove(f)
            print(f"Excluded: {f[0]}")

# Optional: exclude specific PySide6 modules if not needed
# pyside_excludes = [
#     'Qt5WebEngine', 'Qt5Quick', 'Qt5Multimedia', 'Qt5Designer',
#     'Qt5Test', 'Qt5Xml', 'Qt5Sql', 'Qt5Network', 'Qt5DBus',
# ]
# exclude_from_analysis(a, pyside_excludes)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.zipfiles,
    a.datas,
    [],
    name='NuScaler',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=False,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=False,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    icon=None,
) 