
dyn_bin.so:     file format elf64-x86-64


Disassembly of section malloc_hook:

00000000014cfa80 <malloc_hook>:
 14cfa80:	48 85 f6             	test   %rsi,%rsi
 14cfa83:	0f 85 b7 52 bc ff    	jne    1094d40 <VerifyCdmHost_0@@Base+0xe3ba0>
 14cfa89:	55                   	push   %rbp
 14cfa8a:	48 89 e5             	mov    %rsp,%rbp
 14cfa8d:	48 8d 35 ec d5 b5 fe 	lea    -0x14a2a14(%rip),%rsi        # 2d080 <InitializeCdmModule_4@@Base-0xf83e50>
 14cfa94:	48 8d 0d ad 9e b5 fe 	lea    -0x14a6153(%rip),%rcx        # 29948 <InitializeCdmModule_4@@Base-0xf87588>
 14cfa9b:	4c 8d 05 77 9f b5 fe 	lea    -0x14a6089(%rip),%r8        # 29a19 <InitializeCdmModule_4@@Base-0xf874b7>
 14cfaa2:	4c 8d 0d 01 bf b5 fe 	lea    -0x14a40ff(%rip),%r9        # 2b9aa <InitializeCdmModule_4@@Base-0xf85526>
 14cfaa9:	bf 03 00 00 00       	mov    $0x3,%edi
 14cfaae:	ba 6f 02 00 00       	mov    $0x26f,%edx
 14cfab3:	31 c0                	xor    %eax,%eax
 14cfab5:	e8 f6 d9 ba ff       	call   107d4b0 <VerifyCdmHost_0@@Base+0xcc310>
 14cfaba:	0f 0b                	ud2
 14cfabc:	55                   	push   %rbp
 14cfabd:	48 89 e5             	mov    %rsp,%rbp
 14cfac0:	41 57                	push   %r15
 14cfac2:	41 56                	push   %r14
 14cfac4:	53                   	push   %rbx
 14cfac5:	48 81 ec 18 01 00 00 	sub    $0x118,%rsp
 14cfacc:	48 85 ff             	test   %rdi,%rdi
 14cfacf:	0f 84 23 01 00 00    	je     14cfbf8 <VerifyCdmHost_0@@Base+0x51ea58>
 14cfad5:	49 89 fe             	mov    %rdi,%r14
 14cfad8:	48 8b 5f f0          	mov    -0x10(%rdi),%rbx
 14cfadc:	0f 28 05 bd 8a b5 fe 	movaps -0x14a7543(%rip),%xmm0        # 285a0 <InitializeCdmModule_4@@Base-0xf88930>
 14cfae3:	0f 29 85 50 ff ff ff 	movaps %xmm0,-0xb0(%rbp)
 14cfaea:	48 b8 aa aa aa aa aa 	movabs $0xaaaaaaaaaaaaaaaa,%rax
 14cfaf1:	aa aa aa 
 14cfaf4:	48 89 45 d0          	mov    %rax,-0x30(%rbp)
 14cfaf8:	0f 29 45 c0          	movaps %xmm0,-0x40(%rbp)
 14cfafc:	0f 29 45 b0          	movaps %xmm0,-0x50(%rbp)
 14cfb00:	0f 29 45 a0          	movaps %xmm0,-0x60(%rbp)
 14cfb04:	0f 29 45 90          	movaps %xmm0,-0x70(%rbp)
 14cfb08:	0f 29 45 80          	movaps %xmm0,-0x80(%rbp)
 14cfb0c:	0f 29 85 70 ff ff ff 	movaps %xmm0,-0x90(%rbp)
 14cfb13:	0f 29 85 60 ff ff ff 	movaps %xmm0,-0xa0(%rbp)
 14cfb1a:	66 c7 85 50 ff ff ff 	movw   $0x0,-0xb0(%rbp)
 14cfb21:	00 00 
 14cfb23:	48 89 5d d8          	mov    %rbx,-0x28(%rbp)
 14cfb27:	f6 83 24 01 00 00 02 	testb  $0x2,0x124(%rbx)
 14cfb2e:	48 89 df             	mov    %rbx,%rdi
 14cfb31:	74 65                	je     14cfb98 <VerifyCdmHost_0@@Base+0x51e9f8>
 14cfb33:	0f 29 85 40 ff ff ff 	movaps %xmm0,-0xc0(%rbp)
 14cfb3a:	0f 29 85 30 ff ff ff 	movaps %xmm0,-0xd0(%rbp)
 14cfb41:	0f 29 85 20 ff ff ff 	movaps %xmm0,-0xe0(%rbp)
 14cfb48:	0f 29 85 10 ff ff ff 	movaps %xmm0,-0xf0(%rbp)
 14cfb4f:	0f 29 85 00 ff ff ff 	movaps %xmm0,-0x100(%rbp)
 14cfb56:	0f 29 85 f0 fe ff ff 	movaps %xmm0,-0x110(%rbp)
 14cfb5d:	0f 29 85 e0 fe ff ff 	movaps %xmm0,-0x120(%rbp)
 14cfb64:	0f 29 85 d0 fe ff ff 	movaps %xmm0,-0x130(%rbp)
 14cfb6b:	4c 8d bd d0 fe ff ff 	lea    -0x130(%rbp),%r15
 14cfb72:	4c 89 ff             	mov    %r15,%rdi
 14cfb75:	e8 36 02 00 00       	call   14cfdb0 <sigfillset@plt>
 14cfb7a:	48 8d 95 58 ff ff ff 	lea    -0xa8(%rbp),%rdx
 14cfb81:	31 ff                	xor    %edi,%edi
 14cfb83:	4c 89 fe             	mov    %r15,%rsi
 14cfb86:	e8 35 02 00 00       	call   14cfdc0 <pthread_sigmask@plt>
 14cfb8b:	85 c0                	test   %eax,%eax
 14cfb8d:	0f 94 85 51 ff ff ff 	sete   -0xaf(%rbp)
 14cfb94:	48 8b 7d d8          	mov    -0x28(%rbp),%rdi
 14cfb98:	8b 07                	mov    (%rdi),%eax
 14cfb9a:	a8 01                	test   $0x1,%al
 14cfb9c:	75 68                	jne    14cfc06 <VerifyCdmHost_0@@Base+0x51ea66>
 14cfb9e:	89 c1                	mov    %eax,%ecx
 14cfba0:	83 c9 01             	or     $0x1,%ecx
 14cfba3:	f0 0f b1 0f          	lock cmpxchg %ecx,(%rdi)
 14cfba7:	a8 01                	test   $0x1,%al
 14cfba9:	75 5b                	jne    14cfc06 <VerifyCdmHost_0@@Base+0x51ea66>
 14cfbab:	4c 89 f7             	mov    %r14,%rdi
 14cfbae:	48 89 de             	mov    %rbx,%rsi
 14cfbb1:	e8 2a 4f bc ff       	call   1094ae0 <VerifyCdmHost_0@@Base+0xe3940>
 14cfbb6:	8b 83 20 01 00 00    	mov    0x120(%rbx),%eax
 14cfbbc:	85 c0                	test   %eax,%eax
 14cfbbe:	7e 5d                	jle    14cfc1d <VerifyCdmHost_0@@Base+0x51ea7d>
 14cfbc0:	ff c8                	dec    %eax
 14cfbc2:	89 83 20 01 00 00    	mov    %eax,0x120(%rbx)
 14cfbc8:	48 8b 7d d8          	mov    -0x28(%rbp),%rdi
 14cfbcc:	8b 37                	mov    (%rdi),%esi
 14cfbce:	83 e6 02             	and    $0x2,%esi
 14cfbd1:	87 37                	xchg   %esi,(%rdi)
 14cfbd3:	83 fe 08             	cmp    $0x8,%esi
 14cfbd6:	73 35                	jae    14cfc0d <VerifyCdmHost_0@@Base+0x51ea6d>
 14cfbd8:	80 bd 51 ff ff ff 01 	cmpb   $0x1,-0xaf(%rbp)
 14cfbdf:	75 17                	jne    14cfbf8 <VerifyCdmHost_0@@Base+0x51ea58>
 14cfbe1:	48 8d b5 58 ff ff ff 	lea    -0xa8(%rbp),%rsi
 14cfbe8:	bf 02 00 00 00       	mov    $0x2,%edi
 14cfbed:	31 d2                	xor    %edx,%edx
 14cfbef:	e8 cc 01 00 00       	call   14cfdc0 <pthread_sigmask@plt>
 14cfbf4:	85 c0                	test   %eax,%eax
 14cfbf6:	75 54                	jne    14cfc4c <VerifyCdmHost_0@@Base+0x51eaac>
 14cfbf8:	48 81 c4 18 01 00 00 	add    $0x118,%rsp
 14cfbff:	5b                   	pop    %rbx
 14cfc00:	41 5e                	pop    %r14
 14cfc02:	41 5f                	pop    %r15
 14cfc04:	5d                   	pop    %rbp
 14cfc05:	c3                   	ret
 14cfc06:	e8 ab cd ba ff       	call   107c9b6 <VerifyCdmHost_0@@Base+0xcb816>
 14cfc0b:	eb 9e                	jmp    14cfbab <VerifyCdmHost_0@@Base+0x51ea0b>
 14cfc0d:	e8 f6 ce ba ff       	call   107cb08 <VerifyCdmHost_0@@Base+0xcb968>
 14cfc12:	80 bd 51 ff ff ff 01 	cmpb   $0x1,-0xaf(%rbp)
 14cfc19:	74 c6                	je     14cfbe1 <VerifyCdmHost_0@@Base+0x51ea41>
 14cfc1b:	eb db                	jmp    14cfbf8 <VerifyCdmHost_0@@Base+0x51ea58>
 14cfc1d:	48 8d 35 5c d4 b5 fe 	lea    -0x14a2ba4(%rip),%rsi        # 2d080 <InitializeCdmModule_4@@Base-0xf83e50>
 14cfc24:	48 8d 0d 1d 9d b5 fe 	lea    -0x14a62e3(%rip),%rcx        # 29948 <InitializeCdmModule_4@@Base-0xf87588>
 14cfc2b:	4c 8d 05 76 c2 b5 fe 	lea    -0x14a3d8a(%rip),%r8        # 2bea8 <InitializeCdmModule_4@@Base-0xf85028>
 14cfc32:	4c 8d 0d ad b6 b5 fe 	lea    -0x14a4953(%rip),%r9        # 2b2e6 <InitializeCdmModule_4@@Base-0xf85bea>
 14cfc39:	bf 03 00 00 00       	mov    $0x3,%edi
 14cfc3e:	ba 04 02 00 00       	mov    $0x204,%edx
 14cfc43:	31 c0                	xor    %eax,%eax
 14cfc45:	e8 66 d8 ba ff       	call   107d4b0 <VerifyCdmHost_0@@Base+0xcc310>
 14cfc4a:	0f 0b                	ud2
 14cfc4c:	48 8d 35 2d d4 b5 fe 	lea    -0x14a2bd3(%rip),%rsi        # 2d080 <InitializeCdmModule_4@@Base-0xf83e50>
 14cfc53:	48 8d 0d 01 bc b5 fe 	lea    -0x14a43ff(%rip),%rcx        # 2b85b <InitializeCdmModule_4@@Base-0xf85675>
 14cfc5a:	bf 03 00 00 00       	mov    $0x3,%edi
 14cfc5f:	ba 2d 01 00 00       	mov    $0x12d,%edx
 14cfc64:	41 89 c0             	mov    %eax,%r8d
 14cfc67:	31 c0                	xor    %eax,%eax
 14cfc69:	e8 42 d8 ba ff       	call   107d4b0 <VerifyCdmHost_0@@Base+0xcc310>
 14cfc6e:	0f 0b                	ud2
 14cfc70:	55                   	push   %rbp
 14cfc71:	48 89 e5             	mov    %rsp,%rbp
 14cfc74:	53                   	push   %rbx
 14cfc75:	50                   	push   %rax
 14cfc76:	48 89 fb             	mov    %rdi,%rbx
 14cfc79:	8b 05 09 7e 02 00    	mov    0x27e09(%rip),%eax        # 14f7a88 <__tls_get_addr@plt+0x272b8>
 14cfc7f:	3d dd 00 00 00       	cmp    $0xdd,%eax
 14cfc84:	75 15                	jne    14cfc9b <VerifyCdmHost_0@@Base+0x51eafb>
 14cfc86:	48 8d 35 b3 7c 02 00 	lea    0x27cb3(%rip),%rsi        # 14f7940 <__tls_get_addr@plt+0x27170>
 14cfc8d:	48 89 df             	mov    %rbx,%rdi
 14cfc90:	48 83 c4 08          	add    $0x8,%rsp
 14cfc94:	5b                   	pop    %rbx
 14cfc95:	5d                   	pop    %rbp
 14cfc96:	e9 a5 50 bc ff       	jmp    1094d40 <VerifyCdmHost_0@@Base+0xe3ba0>
 14cfc9b:	48 8d 3d e6 7d 02 00 	lea    0x27de6(%rip),%rdi        # 14f7a88 <__tls_get_addr@plt+0x272b8>
 14cfca2:	48 8d 15 57 45 bc ff 	lea    -0x43baa9(%rip),%rdx        # 1094200 <VerifyCdmHost_0@@Base+0xe3060>
 14cfca9:	31 f6                	xor    %esi,%esi
 14cfcab:	e8 00 5a bc ff       	call   10956b0 <VerifyCdmHost_0@@Base+0xe4510>
 14cfcb0:	eb d4                	jmp    14cfc86 <VerifyCdmHost_0@@Base+0x51eae6>
