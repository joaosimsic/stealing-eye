
dyn_bin.so:     file format elf64-x86-64


Disassembly of section .text:

0000000000fb1250 <VerifyCdmHost_0@@Base+0xb0>:
  fb1250:	55                   	push   %rbp
  fb1251:	48 89 e5             	mov    %rsp,%rbp
  fb1254:	48 8d 3d 35 fc ff ff 	lea    -0x3cb(%rip),%rdi        # fb0e90 <InitializeCdmModule_4@@Base-0x40>
  fb125b:	48 8d 35 ce 3d 54 00 	lea    0x543dce(%rip),%rsi        # 14f5030 <__tls_get_addr@plt+0x24860>
  fb1262:	48 8d 15 97 fd 52 00 	lea    0x52fd97(%rip),%rdx        # 14e1000 <__tls_get_addr@plt+0x10830>
  fb1269:	e8 72 ea 51 00       	call   14cfce0 <__cxa_atexit@plt>
  fb126e:	e8 ad fd 3d 00       	call   1391020 <VerifyCdmHost_0@@Base+0x3dfe80>
  fb1273:	5d                   	pop    %rbp
  fb1274:	e9 c7 06 3c 00       	jmp    1371940 <VerifyCdmHost_0@@Base+0x3c07a0>
  fb1279:	cc                   	int3
  fb127a:	cc                   	int3
  fb127b:	cc                   	int3
  fb127c:	cc                   	int3
  fb127d:	cc                   	int3
  fb127e:	cc                   	int3
  fb127f:	cc                   	int3
  fb1280:	55                   	push   %rbp
  fb1281:	48 89 e5             	mov    %rsp,%rbp
  fb1284:	41 56                	push   %r14
  fb1286:	53                   	push   %rbx
  fb1287:	48 83 ec 10          	sub    $0x10,%rsp
  fb128b:	48 89 d3             	mov    %rdx,%rbx
  fb128e:	49 89 fe             	mov    %rdi,%r14
  fb1291:	48 89 37             	mov    %rsi,(%rdi)
  fb1294:	48 89 f7             	mov    %rsi,%rdi
  fb1297:	e8 44 24 01 00       	call   fc36e0 <VerifyCdmHost_0@@Base+0x12540>
  fb129c:	66 0f 57 c9          	xorpd  %xmm1,%xmm1
  fb12a0:	66 0f 2e c1          	ucomisd %xmm1,%xmm0
  fb12a4:	74 5e                	je     fb1304 <VerifyCdmHost_0@@Base+0x164>
  fb12a6:	f2 0f 59 05 ea 61 07 	mulsd  -0xf89e16(%rip),%xmm0        # 27498 <InitializeCdmModule_4@@Base-0xf89a38>
  fb12ad:	ff 
  fb12ae:	31 d2                	xor    %edx,%edx
  fb12b0:	f2 0f 10 0d 88 61 07 	movsd  -0xf89e78(%rip),%xmm1        # 27440 <InitializeCdmModule_4@@Base-0xf89a90>
  fb12b7:	ff 
  fb12b8:	66 0f 2e c1          	ucomisd %xmm1,%xmm0
  fb12bc:	48 b9 ff ff ff ff ff 	movabs $0x7fffffffffffffff,%rcx
  fb12c3:	ff ff 7f 
  fb12c6:	48 0f 43 d1          	cmovae %rcx,%rdx
  fb12ca:	f2 0f 10 15 c6 5e 07 	movsd  -0xf8a13a(%rip),%xmm2        # 27198 <InitializeCdmModule_4@@Base-0xf89d38>
  fb12d1:	ff 
  fb12d2:	66 0f 2e d0          	ucomisd %xmm0,%xmm2
  fb12d6:	f2 0f 5f c1          	maxsd  %xmm1,%xmm0
  fb12da:	f2 48 0f 2c c0       	cvttsd2si %xmm0,%rax
  fb12df:	48 0f 42 c2          	cmovb  %rdx,%rax
  fb12e3:	48 8d 14 08          	lea    (%rax,%rcx,1),%rdx
  fb12e7:	48 83 c2 02          	add    $0x2,%rdx
  fb12eb:	48 83 fa 02          	cmp    $0x2,%rdx
  fb12ef:	72 15                	jb     fb1306 <VerifyCdmHost_0@@Base+0x166>
  fb12f1:	48 ba 00 40 86 48 96 	movabs $0x295e9648864000,%rdx
  fb12f8:	5e 29 00 
  fb12fb:	48 01 d0             	add    %rdx,%rax
  fb12fe:	48                   	rex.W
  fb12ff:	0f                   	.byte 0xf
