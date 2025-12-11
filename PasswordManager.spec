# -*- mode: python ; coding: utf-8 -*-


a = Analysis(
    ['interface/page_1.py'],
    pathex=[],
    binaries=[('vault_core/target/debug/release/libvault_core.so', 'vault_core')],
    datas=[('gui', 'gui'), ('vault_core/src/storage.json', 'vault_core/src')],
    hiddenimports=[],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    noarchive=False,
    optimize=0,
)
pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.datas,
    [],
    name='PasswordManager',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=False,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
