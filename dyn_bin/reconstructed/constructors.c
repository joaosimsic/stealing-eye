/*
 * dyn_bin.so — .init_array constructor inventory.
 *
 * The section is all-zero in the file; the loader fills it from
 * .rela.dyn R_X86_64_RELATIVE entries (see ../relocations.txt). Order below
 * is file order in .init_array (0x14d1848 .. 0x14d1888).
 *
 * Artifact: /home/joao/projects/stealing-eye/dyn_bin.so
 * SHA-256:  5a34d8576612cb07e1c19c2203a8b7dbb5521324ceb3fea8131d84648aea9902
 */

/* .init_array slot -> target vaddr */
const unsigned long kInitArray[] = {
    0x14c9570, /* not individually analyzed */
    0x14a58d0, /* not individually analyzed */
    0x14cc780, /* not individually analyzed */
    0xe0eeb0,  /* 3-byte thunk: jmp 0xe0ee30 (in .text prologue area) */
    0xfb1250,  /* module destructor registration, see below */
    0xffd2d0,  /* not individually analyzed */
    0x101c910, /* not individually analyzed */
    0x10346e0, /* not individually analyzed */
    0x1364f50, /* not individually analyzed */
};

/*
 * 0xfb1250 — constructor #5 (disassembly: ../disasm/ctor_fb1250.asm)
 *
 *   push rbp / mov rsp,rbp
 *   lea rdi, [0xfb0e90]        ; destructor thunk (wraps module teardown)
 *   lea rsi, [0x14f5030]       ; &g_cdm_module (module singleton, .bss)
 *   lea rdx, [0x14e1000]       ; __dso_handle (slot in .data)
 *   call __cxa_atexit          ; register destructor for the DSO
 *   call 0x1391020             ; init routine (logging/crypto subsystem)
 *   jmp  0x1371940             ; tail call, further init
 *
 * So merely dlopen()-ing this library registers a process-exit-time
 * destructor for the module singleton and initializes internal subsystems.
 */
