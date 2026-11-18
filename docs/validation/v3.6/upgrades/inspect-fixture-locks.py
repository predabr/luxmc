import ctypes
import json
import os
from ctypes import wintypes
from pathlib import Path

root = Path(os.environ['APPDATA']) / 'github/Luxmc/data/instances/639d81a1-950d-4419-bdac-be0b3c8f25cb'
kernel = ctypes.WinDLL('kernel32', use_last_error=True)
kernel.CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD, ctypes.c_void_p, wintypes.DWORD, wintypes.DWORD, wintypes.HANDLE]
kernel.CreateFileW.restype = wintypes.HANDLE
kernel.CloseHandle.argtypes = [wintypes.HANDLE]
blocked = []
for path in [root, *root.rglob('*')]:
    handle = kernel.CreateFileW(str(path), 0x10000, 7, None, 3, 0x02000000, None)
    if handle == ctypes.c_void_p(-1).value:
        code = ctypes.get_last_error()
        if code == 32:
            blocked.append(str(path))
    else:
        kernel.CloseHandle(handle)

class UniqueProcess(ctypes.Structure):
    _fields_ = [('pid', wintypes.DWORD), ('start', wintypes.FILETIME)]

class ProcessInfo(ctypes.Structure):
    _fields_ = [('process', UniqueProcess), ('name', wintypes.WCHAR * 256), ('service', wintypes.WCHAR * 64), ('type', wintypes.DWORD), ('status', wintypes.ULONG), ('session', wintypes.DWORD), ('restartable', wintypes.BOOL)]

owners = []
files = [path for path in blocked if Path(path).is_file()]
if files:
    manager = ctypes.WinDLL('Rstrtmgr')
    session = wintypes.DWORD()
    key = ctypes.create_unicode_buffer(33)
    assert manager.RmStartSession(ctypes.byref(session), 0, key) == 0
    try:
        resources = (wintypes.LPCWSTR * len(files))(*files)
        assert manager.RmRegisterResources(session, len(files), resources, 0, None, 0, None) == 0
        needed, count, reason = wintypes.UINT(), wintypes.UINT(), wintypes.DWORD()
        status = manager.RmGetList(session, ctypes.byref(needed), ctypes.byref(count), None, ctypes.byref(reason))
        if status == 234:
            count.value = needed.value
            processes = (ProcessInfo * needed.value)()
            status = manager.RmGetList(session, ctypes.byref(needed), ctypes.byref(count), processes, ctypes.byref(reason))
            assert status == 0
            owners = [{'pid': info.process.pid, 'name': info.name} for info in processes[:count.value]]
    finally:
        manager.RmEndSession(session)

print(json.dumps({'blockedPaths': blocked, 'owners': owners}, ensure_ascii=False, indent=2))
