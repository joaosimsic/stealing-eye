/*
 * Reconstruction of dyn_bin.so public ABI (Widevine CDM 4.10.3112.0).
 *
 * Source: GNU objdump disassembly of the 5 exported functions (see
 * ../disasm/exports.asm). This is analyst-reconstructed pseudocode, not
 * original source. Vaddrs are ELF virtual addresses; .text file offset is
 * vaddr - 0x1000. Helper functions (with 0x... names) were not decompiled;
 * their roles are inferred from calling convention and usage.
 *
 * Artifact: /home/joao/projects/stealing-eye/dyn_bin.so
 * SHA-256:  5a34d8576612cb07e1c19c2203a8b7dbb5521324ceb3fea8131d84648aea9902
 */

#include <stdint.h>
#include <stddef.h>

/* ------------------------------------------------------------------ */
/* Imports observed in .dynsym (PLT targets used below)                */
/* ------------------------------------------------------------------ */
extern void *operator_new(size_t);           /* 0x137c350 */
extern void  operator_delete(void *);        /* 0x137c3a0 */

/* Helper callees (internal, stripped). Roles inferred. */
extern void  ctor_1364f90(void *);           /* 0x1364f90 */
extern void  dtor_1364ff0(void *);           /* 0x1364ff0 */
extern void *singleton_107c8a0(void);        /* 0x107c8a0 */
extern void  init_107c800(void *);           /* 0x107c800  (tail-called) */
extern void  logging_107c870(void *);        /* 0x107c870 */
extern void  init_101adf0(void);             /* 0x101adf0 */
extern void  fini_ffd1d0(void);              /* 0xffd1d0 */

/* Host interface factory provided by the embedder (Chrome). */
typedef void *(*GetCdmHostFunc)(int cdm_interface_version, void *user_data);

/* Callback thunks used to build the internal dispatcher object. */
extern void fn_fb11b0(void);                 /* 0xfb11b0 */
extern void fn_fb1210(void);                 /* 0xfb1210 */

/* Constructors for internal objects. */
extern void ctor_fabe00(void *self, void *host);        /* 0x40-byte host adapter, v10 */
extern void ctor_face10(void *self, void *host);        /* 0x40-byte host adapter, v11 */
extern void ctor_1366f80(void *self, void *a, void *b); /* 0x38-byte dispatcher */
extern void ctor_101d410(void *self, void *dispatcher); /* 0x30-byte state */
extern void local_scope_1367030(void *self); /* 0x1367030 */
extern void *fn_fca370(void *host_adapter, void *state); /* 0xfca370 */
extern void ctor_fb64b0(void *self, void *host_adapter,
                        void *state, void *inner);       /* 0xfb64b0 */

/* ------------------------------------------------------------------ */
/* Module state                                                        */
/* ------------------------------------------------------------------ */

/* .bss:0x14f5030 — holds the CDM module singleton (0x50 bytes). */
static void *g_cdm_module;

/* .rodata:0x2be9c — Widevine CDM version string, NUL-terminated. */
static const char kCdmVersion[] = "4.10.3112.0";

/* ------------------------------------------------------------------ */
/* Exports                                                             */
/* ------------------------------------------------------------------ */

/* 0xfb11a0, 8 bytes */
int VerifyCdmHost_0(void)
{
    return 1; /* mov $0x1,%al; ret — host verification is a no-op in this build */
}

/* 0xfb1190, 13 bytes */
const char *GetCdmVersion(void)
{
    return kCdmVersion;
}

/* 0xfb0ed0, 90 bytes */
void InitializeCdmModule_4(void)
{
    if (g_cdm_module == NULL) {
        void *p = operator_new(0x50);
        ctor_1364f90(p);
        void *old = g_cdm_module;
        g_cdm_module = p;
        if (old != NULL) {
            dtor_1364ff0(old);
            operator_delete(old);
        }
    }
    /* tail call: init_107c800(singleton_107c8a0()) — logging/singleton init */
    init_107c800(singleton_107c8a0());
}

/* 0xfb0f30, 81 bytes */
void DeinitializeCdmModule(void)
{
    init_101adf0();
    fini_ffd1d0();
    logging_107c870(singleton_107c8a0());

    void *old = g_cdm_module;
    g_cdm_module = NULL;
    if (old != NULL) {
        dtor_1364ff0(old);
        operator_delete(old);
    }
}

/* 0xfb0f90, 511 bytes.
 *
 * Documented Widevine loader ABI:
 *   void* CreateCdmInstance(int cdm_interface_version,
 *                           const char* key_system, uint32_t key_system_size,
 *                           GetCdmHostFunc get_cdm_host, void* user_data);
 *
 * key_system/key_system_size are not examined in this export body
 * (validation, if any, happens inside ctor_fb64b0); only versions 10 and 11
 * are accepted. Observed shape per version:
 *   1. host = get_cdm_host(version, user_data);           NULL => NULL
 *   2. host_adapter = new(0x40); ctor_{fabe00|face10}(host_adapter, host)
 *   3. dispatcher   = new(0x38); ctor_1366f80(dispatcher, &fn_fb11b0, &fn_fb1210)
 *        dispatcher[0x20] = 0x19 (25)
 *        dispatcher[0x28] = 0
 *        dispatcher[0x30] = host_adapter
 *   4. state        = new(0x30); ctor_101d410(state, dispatcher)
 *   5. inner        = fn_fca370(host_adapter, state)
 *   6. instance     = new(0x18); ctor_fb64b0(instance, host_adapter, state, inner)
 *
 * Return value:
 *   version 10 -> instance
 *   version 11 -> (char*)instance + 8  (interface sub-object at offset 8),
 *                 or NULL when instance is NULL
 */
void *CreateCdmInstance(int cdm_interface_version, const char *key_system,
                        uint32_t key_system_size, GetCdmHostFunc get_cdm_host,
                        void *user_data)
{
    (void)key_system;
    (void)key_system_size;

    if (cdm_interface_version == 11) {
        void *host = get_cdm_host(11, user_data);
        if (host == NULL)
            return NULL;

        void *host_adapter = operator_new(0x40);
        ctor_face10(host_adapter, host);

        /* PartitionAlloc pointer-tag check against a global pool at
         * .data:0x14e2000 (if (ptr & 0xfffffffc00000000) == pool) ... */
#if 0
        if (((uintptr_t)host_adapter & 0xfffffffc00000000ULL) == g_some_pool) {
            fn_1373e70(host_adapter); /* 0x1373e70, role unknown */
        }
#endif
        void *dispatcher = operator_new(0x38);
        ctor_1366f80(dispatcher, (void *)fn_fb11b0, (void *)fn_fb1210);
        *(uint64_t *)((char *)dispatcher + 0x20) = 0x19;
        *(uint64_t *)((char *)dispatcher + 0x28) = 0;
        *(void **)((char *)dispatcher + 0x30) = host_adapter;

        void *state = operator_new(0x30);
        ctor_101d410(state, dispatcher);
        /* stack local at rbp-0x28 zeroed, then 0x1367030(&local) is called;
         * a scope object (lock/guard) constructed and torn down here */
        uint64_t local = 0;
        local_scope_1367030(&local);

        void *inner = fn_fca370(host_adapter, state);

        void *instance = operator_new(0x18);
        ctor_fb64b0(instance, host_adapter, state, inner);
        return instance ? (char *)instance + 8 : NULL;
    }

    if (cdm_interface_version == 10) {
        void *host = get_cdm_host(10, user_data);
        if (host == NULL)
            return NULL;

        void *host_adapter = operator_new(0x40);
        ctor_fabe00(host_adapter, host);

        /* same PartitionAlloc tag check as above (0x1373e70) */
        void *dispatcher = operator_new(0x38);
        ctor_1366f80(dispatcher, (void *)fn_fb11b0, (void *)fn_fb1210);
        *(uint64_t *)((char *)dispatcher + 0x20) = 0x19;
        *(uint64_t *)((char *)dispatcher + 0x28) = 0;
        *(void **)((char *)dispatcher + 0x30) = host_adapter;

        void *state = operator_new(0x30);
        ctor_101d410(state, dispatcher);
        uint64_t local = 0;
        local_scope_1367030(&local);

        void *inner = fn_fca370(host_adapter, state);

        void *instance = operator_new(0x18);
        ctor_fb64b0(instance, host_adapter, state, inner);
        return instance;
    }

    return NULL; /* unsupported cdm_interface_version */
}
