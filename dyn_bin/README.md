# dyn_bin.so — reverse-engineering notes

## Verdict

`dyn_bin.so` is the **Google Widevine CDM** ("Content Decryption Module") — the DRM
plugin Chrome loads for Widevine-protected media. It is a plain shared library,
loaded by the browser via `dlopen`; it spawns no process and its own code performs
no network I/O (the host browser provides network and storage through callbacks).

- CDM version: `4.10.3112.0` (returned by `GetCdmVersion`, string at vaddr `0x2be9c`)
- Original library name still embedded: `libwidevinecdm.so`
- `SONAME` was rewritten to `REAtest` (fixture rename; not the original name)
- Not related to the Rust screen recorder in this repo (that project links
  libpulse/X11; this file has neither, and contains no `src/*.rs` strings)

## Identity

| Field | Value |
| --- | --- |
| Path | `/home/joao/projects/stealing-eye/dyn_bin.so` |
| SHA-256 | `5a34d8576612cb07e1c19c2203a8b7dbb5521324ceb3fea8131d84648aea9902` |
| Size | 21974097 bytes |
| Format | ELF64, little-endian, x86-64, `DYN` (shared object), entry `0x0` |
| `NEEDED` | `libdl.so.2`, `libpthread.so.0`, `libm.so.6`, `libgcc_s.so.1`, `libc.so.6`, `ld-linux-x86-64.so.2` |
| `SONAME` | `REAtest` |

## Build provenance (`.comment`, `/rustc` absent, no `.dep-v0`)

- `GCC: (Debian 10.2.1-6) 10.2.1 20210110`
- `rustc version 1.88.0-dev (4a0969e06dbeaaa43914d2d00b2e843d49aa3886-1-llvmorg-21-init-11777-gfd3fecfc chromium)`
- `Linker: LLD 21.0.0 (https://chromium.googlesource.com/a/external/github.com/llvm/llvm-project fd3fecfc0936703f2715fe6fea890e81b0b3c2ac)`

Matches a Chromium/Chrome component build (Chromium's fork of rustc + bundled
libc++ + LLD). The binary is **gcov-instrumented** (`__gcov_dump`, `__gcov_flush`
imports) and carries an external-DWARF debug-info hash
`43a14f80a77ba70684300b0f9e8fbc7cc313b334` (debug file not present).

## ELF layout (see `sections.txt`)

| Section | Vaddr | Size | Notes |
| --- | --- | --- | --- |
| `.rodata` | `0x1d000` | `0xde9510` | merged strings + constants |
| `.text` | `0xe0ee00` | `0x6c0c80` | file offset = vaddr − `0x1000` |
| `malloc_hook` | `0x14cfa80` | `0x232` | executable; Chromium **PartitionAlloc** shims |
| `.tdata`/`.tbss` | `0x14d17e0` | `0x60` | TLS |
| `.bss` | `0x14f5000` | `0x3b088ad` | ~62 MB of zero-init state |

`malloc_hook` disassembly (`disasm/malloc_hook.asm`) shows PartitionAlloc
machinery: `arena->allocation_count > 0`, "nothing in arena to free" checks and a
`std::__Cr` (Chromium libc++ namespace) CHECK-FAIL handler. This is the normal
Chrome allocator shim, not malware.

## Exported ABI (`.dynsym`, see `exports.txt`)

| Address | Size | Symbol |
| --- | --- | --- |
| `0xfb0ed0` | 90 | `InitializeCdmModule_4` |
| `0xfb0f30` | 81 | `DeinitializeCdmModule` |
| `0xfb0f90` | 511 | `CreateCdmInstance` |
| `0xfb1190` | 13 | `GetCdmVersion` |
| `0xfb11a0` | 8 | `VerifyCdmHost_0` |

Plus weak, undefined, host-supplied allocator hooks:
`OPENSSL_memory_alloc`, `OPENSSL_memory_free`, `OPENSSL_memory_get_size`
(Chrome provides these at load time so the CDM can use the browser's OpenSSL).

Reconstructed pseudocode: `reconstructed/exports.c`. Raw disassembly:
`disasm/exports.asm`.

Behavior summary:

- `GetCdmVersion()` → `"4.10.3112.0"` (`0x2be9c`)
- `VerifyCdmHost_0()` → `true`
- `InitializeCdmModule_4()` / `DeinitializeCdmModule()` manage one global heap
  object at `.bss:0x14f5030` (allocated with `operator new(0x50)`) and run the
  logging/singleton init (`0x107c8a0`/`0x107c800`), i.e. standard module
  init/teardown entry points.
- `CreateCdmInstance(version, key_system, key_system_size, get_cdm_host, user_data)`
  accepts interface **version 10 or 11** (otherwise returns `NULL`):
  1. calls the host-provided `get_cdm_host(version, user_data)` factory;
  2. wraps the returned `CdmHost` in a versioned adapter (`0x40`-byte object,
     ctor `0xfabe00` for v10 / `0xface10` for v11);
  3. builds a callback/dispatcher object (`0x38` bytes, callbacks `0xfb11b0`,
     `0xfb1210`) and a `0x30`-byte state object (`ctor 0x101d410`);
  4. creates the interface instance (`0x18` bytes, `ctor 0xfb64b0`).
  For version 11 the returned pointer is the instance **+8** (interface
  sub-object adjustment); version 10 returns the instance pointer directly.

This is exactly the documented Widevine CDM loader ABI
(`InitializeCdmModule_4` / `CreateCdmInstance` / `DeinitializeCdmModule`).

## Constructors (`.init_array`, 9 entries; `reconstructed/constructors.c`)

`0x14c9570`, `0x14a58d0`, `0x14cc780`, `0xe0eeb0` (jmp → `0xe0ee30`),
`0xfb1250`, `0xffd2d0`, `0x101c910`, `0x10346e0`, `0x1364f50`.

`0xfb1250` registers the module destructor:
`__cxa_atexit(dtor_0xfb0e90, &g_module_0x14f5030, __dso_handle)`, then runs
two init routines (`0x1391020`, tail-call `0x1371940`) — logging / ICU-style init.

## Import surface (capability view, `imports.txt`)

- File I/O: `open64`, `creat64`, `fopen64`, `pread64`, `read`, `write`, `fclose`, `__xstat64`
- Memory: `mmap64`, `mprotect`, `munmap`, `madvise`, `mem*`
- Threads/sync: full `pthread_*`, `sem_*`, atomics via libc
- Process info: `getpid`, `geteuid`, `uname`, `prctl`, `syscall` (allocator/thread support)
- Time/locale: `clock_gettime`, `gettimeofday`, `*locale*`, `strftime_l`

**Not present in the dynamic symbol table:** `socket`, `connect`, `send*`, `recv*`,
`exec*`, `fork`, `ptrace`. The CDM itself opens no sockets and starts no
processes; playback traffic/storage is done by the Chrome host through the
`CdmHost` callbacks supplied to `CreateCdmInstance`.

## Strings / indicators (full dump: `strings.txt`, 26,924 unique)

- `libwidevinecdm.so`, `REAtest`, `4.10.3112.0`
- Chromium trace categories (`disabled-by-default-*`, 106 entries)
- `//components/performance_manager/scenario_api/performance_scenarios.h`
- `Actions on Google Hardware, used in Google-internal code.` (vaddr `0x2c3b7`)
- PartitionAlloc check strings: `arena != nullptr`, `nothing in arena to free`,
  `Check %s failed: %s`
- LLVM build URL in `.comment` (see above)

## Method, evidence and limitations

- Tools: `readelf` (headers, sections, dynsym, relocs), GNU `objdump` disassembly,
  REA MCP (`open_binary` target binding, `inspect_rust_binary`, `inspect_with_objdump`).
- Ghidra 12.1.2 (REA provider) **failed to import the file**: Java heap exhaustion
  during auto-analysis (`X86Analyzer`/`DecompilerSwitchAnalyzer` OOM on the 21 MB
  image), so no provider decompiler output exists for this run. All reconstruction
  above is from raw disassembly. To get decompiled functions, rerun with a larger
  Ghidra heap and/or `REA_GHIDRA_DISABLED_ANALYZERS` skipping those analyzers.
- Symbols beyond `.dynsym` are stripped; internal call targets are shown as
  offsets (e.g. `VerifyCdmHost_0+0x3cb1b0`). Helper semantics (constructors
  `0xfabe00`/`0xface10`/`0xfb64b0`, singleton init `0x107c8a0`) are inferred from
  calling convention and surrounding code, not decompiled — treat as derived.
- The `key_system` argument to `CreateCdmInstance` is not validated inside the
  export body (likely checked inside `0xfb64b0`); unverified.

## Files

| File | Content |
| --- | --- |
| `README.md` | this report |
| `SHA256.txt` | artifact digest |
| `exports.txt` / `imports.txt` | defined / undefined dynamic symbols |
| `sections.txt` / `relocations.txt` | ELF section table, all 4524 relocations |
| `strings.txt` / `strings-keywords.txt` | printable-string inventory, filtered view |
| `disasm/exports.asm` | annotated disassembly of the 5 exports + helper thunks |
| `disasm/ctor_fb1250.asm` | module-destructor registering constructor |
| `disasm/malloc_hook.asm` | PartitionAlloc shim section |
| `reconstructed/exports.c` | C reconstruction of the public ABI |
| `reconstructed/constructors.c` | `.init_array` constructor inventory |
