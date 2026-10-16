
dyn_bin.so:     file format elf64-x86-64


Disassembly of section .text:

0000000000fb0e90 <InitializeCdmModule_4@@Base-0x40>:
  fb0e90:	55                   	push   %rbp
  fb0e91:	48 89 e5             	mov    %rsp,%rbp
  fb0e94:	53                   	push   %rbx
  fb0e95:	50                   	push   %rax
  fb0e96:	48 8b 1f             	mov    (%rdi),%rbx
  fb0e99:	48 c7 07 00 00 00 00 	movq   $0x0,(%rdi)
  fb0ea0:	48 85 db             	test   %rbx,%rbx
  fb0ea3:	74 16                	je     fb0ebb <InitializeCdmModule_4@@Base-0x15>
  fb0ea5:	48 89 df             	mov    %rbx,%rdi
  fb0ea8:	e8 43 41 3b 00       	call   1364ff0 <VerifyCdmHost_0@@Base+0x3b3e50>
  fb0ead:	48 89 df             	mov    %rbx,%rdi
  fb0eb0:	48 83 c4 08          	add    $0x8,%rsp
  fb0eb4:	5b                   	pop    %rbx
  fb0eb5:	5d                   	pop    %rbp
  fb0eb6:	e9 e5 b4 3c 00       	jmp    137c3a0 <VerifyCdmHost_0@@Base+0x3cb200>
  fb0ebb:	48 83 c4 08          	add    $0x8,%rsp
  fb0ebf:	5b                   	pop    %rbx
  fb0ec0:	5d                   	pop    %rbp
  fb0ec1:	c3                   	ret
  fb0ec2:	cc                   	int3
  fb0ec3:	cc                   	int3
  fb0ec4:	cc                   	int3
  fb0ec5:	cc                   	int3
  fb0ec6:	cc                   	int3
  fb0ec7:	cc                   	int3
  fb0ec8:	cc                   	int3
  fb0ec9:	cc                   	int3
  fb0eca:	cc                   	int3
  fb0ecb:	cc                   	int3
  fb0ecc:	cc                   	int3
  fb0ecd:	cc                   	int3
  fb0ece:	cc                   	int3
  fb0ecf:	cc                   	int3

0000000000fb0ed0 <InitializeCdmModule_4@@Base>:
  fb0ed0:	55                   	push   %rbp
  fb0ed1:	48 89 e5             	mov    %rsp,%rbp
  fb0ed4:	41 56                	push   %r14
  fb0ed6:	53                   	push   %rbx
  fb0ed7:	48 83 3d 51 41 54 00 	cmpq   $0x0,0x544151(%rip)        # 14f5030 <__tls_get_addr@plt+0x24860>
  fb0ede:	00 
  fb0edf:	75 38                	jne    fb0f19 <InitializeCdmModule_4@@Base+0x49>
  fb0ee1:	bf 50 00 00 00       	mov    $0x50,%edi
  fb0ee6:	e8 65 b4 3c 00       	call   137c350 <VerifyCdmHost_0@@Base+0x3cb1b0>
  fb0eeb:	49 89 c6             	mov    %rax,%r14
  fb0eee:	48 89 c7             	mov    %rax,%rdi
  fb0ef1:	e8 9a 40 3b 00       	call   1364f90 <VerifyCdmHost_0@@Base+0x3b3df0>
  fb0ef6:	48 8b 1d 33 41 54 00 	mov    0x544133(%rip),%rbx        # 14f5030 <__tls_get_addr@plt+0x24860>
  fb0efd:	4c 89 35 2c 41 54 00 	mov    %r14,0x54412c(%rip)        # 14f5030 <__tls_get_addr@plt+0x24860>
  fb0f04:	48 85 db             	test   %rbx,%rbx
  fb0f07:	74 10                	je     fb0f19 <InitializeCdmModule_4@@Base+0x49>
  fb0f09:	48 89 df             	mov    %rbx,%rdi
  fb0f0c:	e8 df 40 3b 00       	call   1364ff0 <VerifyCdmHost_0@@Base+0x3b3e50>
  fb0f11:	48 89 df             	mov    %rbx,%rdi
  fb0f14:	e8 87 b4 3c 00       	call   137c3a0 <VerifyCdmHost_0@@Base+0x3cb200>
  fb0f19:	e8 82 b9 0c 00       	call   107c8a0 <VerifyCdmHost_0@@Base+0xcb700>
  fb0f1e:	48 89 c7             	mov    %rax,%rdi
  fb0f21:	5b                   	pop    %rbx
  fb0f22:	41 5e                	pop    %r14
  fb0f24:	5d                   	pop    %rbp
  fb0f25:	e9 d6 b8 0c 00       	jmp    107c800 <VerifyCdmHost_0@@Base+0xcb660>
  fb0f2a:	cc                   	int3
  fb0f2b:	cc                   	int3
  fb0f2c:	cc                   	int3
  fb0f2d:	cc                   	int3
  fb0f2e:	cc                   	int3
  fb0f2f:	cc                   	int3

0000000000fb0f30 <DeinitializeCdmModule@@Base>:
  fb0f30:	55                   	push   %rbp
  fb0f31:	48 89 e5             	mov    %rsp,%rbp
  fb0f34:	53                   	push   %rbx
  fb0f35:	50                   	push   %rax
  fb0f36:	e8 b5 9e 06 00       	call   101adf0 <VerifyCdmHost_0@@Base+0x69c50>
  fb0f3b:	e8 90 c2 04 00       	call   ffd1d0 <VerifyCdmHost_0@@Base+0x4c030>
  fb0f40:	e8 5b b9 0c 00       	call   107c8a0 <VerifyCdmHost_0@@Base+0xcb700>
  fb0f45:	48 89 c7             	mov    %rax,%rdi
  fb0f48:	e8 23 b9 0c 00       	call   107c870 <VerifyCdmHost_0@@Base+0xcb6d0>
  fb0f4d:	48 8b 1d dc 40 54 00 	mov    0x5440dc(%rip),%rbx        # 14f5030 <__tls_get_addr@plt+0x24860>
  fb0f54:	48 c7 05 d1 40 54 00 	movq   $0x0,0x5440d1(%rip)        # 14f5030 <__tls_get_addr@plt+0x24860>
  fb0f5b:	00 00 00 00 
  fb0f5f:	48 85 db             	test   %rbx,%rbx
  fb0f62:	74 16                	je     fb0f7a <DeinitializeCdmModule@@Base+0x4a>
  fb0f64:	48 89 df             	mov    %rbx,%rdi
  fb0f67:	e8 84 40 3b 00       	call   1364ff0 <VerifyCdmHost_0@@Base+0x3b3e50>
  fb0f6c:	48 89 df             	mov    %rbx,%rdi
  fb0f6f:	48 83 c4 08          	add    $0x8,%rsp
  fb0f73:	5b                   	pop    %rbx
  fb0f74:	5d                   	pop    %rbp
  fb0f75:	e9 26 b4 3c 00       	jmp    137c3a0 <VerifyCdmHost_0@@Base+0x3cb200>
  fb0f7a:	48 83 c4 08          	add    $0x8,%rsp
  fb0f7e:	5b                   	pop    %rbx
  fb0f7f:	5d                   	pop    %rbp
  fb0f80:	c3                   	ret
  fb0f81:	cc                   	int3
  fb0f82:	cc                   	int3
  fb0f83:	cc                   	int3
  fb0f84:	cc                   	int3
  fb0f85:	cc                   	int3
  fb0f86:	cc                   	int3
  fb0f87:	cc                   	int3
  fb0f88:	cc                   	int3
  fb0f89:	cc                   	int3
  fb0f8a:	cc                   	int3
  fb0f8b:	cc                   	int3
  fb0f8c:	cc                   	int3
  fb0f8d:	cc                   	int3
  fb0f8e:	cc                   	int3
  fb0f8f:	cc                   	int3

0000000000fb0f90 <CreateCdmInstance@@Base>:
  fb0f90:	55                   	push   %rbp
  fb0f91:	48 89 e5             	mov    %rsp,%rbp
  fb0f94:	41 57                	push   %r15
  fb0f96:	41 56                	push   %r14
  fb0f98:	41 54                	push   %r12
  fb0f9a:	53                   	push   %rbx
  fb0f9b:	48 83 ec 10          	sub    $0x10,%rsp
  fb0f9f:	83 ff 0b             	cmp    $0xb,%edi
  fb0fa2:	0f 84 ea 00 00 00    	je     fb1092 <CreateCdmInstance@@Base+0x102>
  fb0fa8:	83 ff 0a             	cmp    $0xa,%edi
  fb0fab:	0f 85 cb 01 00 00    	jne    fb117c <CreateCdmInstance@@Base+0x1ec>
  fb0fb1:	bf 0a 00 00 00       	mov    $0xa,%edi
  fb0fb6:	4c 89 c6             	mov    %r8,%rsi
  fb0fb9:	ff d1                	call   *%rcx
  fb0fbb:	48 85 c0             	test   %rax,%rax
  fb0fbe:	0f 84 b8 01 00 00    	je     fb117c <CreateCdmInstance@@Base+0x1ec>
  fb0fc4:	49 89 c6             	mov    %rax,%r14
  fb0fc7:	bf 40 00 00 00       	mov    $0x40,%edi
  fb0fcc:	e8 7f b3 3c 00       	call   137c350 <VerifyCdmHost_0@@Base+0x3cb1b0>
  fb0fd1:	48 89 c3             	mov    %rax,%rbx
  fb0fd4:	48 89 c7             	mov    %rax,%rdi
  fb0fd7:	4c 89 f6             	mov    %r14,%rsi
  fb0fda:	e8 21 ae ff ff       	call   fabe00 <InitializeCdmModule_4@@Base-0x50d0>
  fb0fdf:	48 b8 00 00 00 00 fc 	movabs $0xfffffffc00000000,%rax
  fb0fe6:	ff ff ff 
  fb0fe9:	48 21 d8             	and    %rbx,%rax
  fb0fec:	48 8d 0d 0d 10 53 00 	lea    0x53100d(%rip),%rcx        # 14e2000 <__tls_get_addr@plt+0x11830>
  fb0ff3:	48 3b 41 08          	cmp    0x8(%rcx),%rax
  fb0ff7:	75 08                	jne    fb1001 <CreateCdmInstance@@Base+0x71>
  fb0ff9:	48 89 df             	mov    %rbx,%rdi
  fb0ffc:	e8 6f 2e 3c 00       	call   1373e70 <VerifyCdmHost_0@@Base+0x3c2cd0>
  fb1001:	bf 38 00 00 00       	mov    $0x38,%edi
  fb1006:	e8 45 b3 3c 00       	call   137c350 <VerifyCdmHost_0@@Base+0x3cb1b0>
  fb100b:	49 89 c6             	mov    %rax,%r14
  fb100e:	48 8d 35 9b 01 00 00 	lea    0x19b(%rip),%rsi        # fb11b0 <VerifyCdmHost_0@@Base+0x10>
  fb1015:	48 8d 15 f4 01 00 00 	lea    0x1f4(%rip),%rdx        # fb1210 <VerifyCdmHost_0@@Base+0x70>
  fb101c:	48 89 c7             	mov    %rax,%rdi
  fb101f:	e8 5c 5f 3b 00       	call   1366f80 <VerifyCdmHost_0@@Base+0x3b5de0>
  fb1024:	49 c7 46 20 19 00 00 	movq   $0x19,0x20(%r14)
  fb102b:	00 
  fb102c:	49 c7 46 28 00 00 00 	movq   $0x0,0x28(%r14)
  fb1033:	00 
  fb1034:	49 89 5e 30          	mov    %rbx,0x30(%r14)
  fb1038:	bf 30 00 00 00       	mov    $0x30,%edi
  fb103d:	e8 0e b3 3c 00       	call   137c350 <VerifyCdmHost_0@@Base+0x3cb1b0>
  fb1042:	49 89 c7             	mov    %rax,%r15
  fb1045:	48 c7 45 d8 00 00 00 	movq   $0x0,-0x28(%rbp)
  fb104c:	00 
  fb104d:	48 89 c7             	mov    %rax,%rdi
  fb1050:	4c 89 f6             	mov    %r14,%rsi
  fb1053:	e8 b8 c3 06 00       	call   101d410 <VerifyCdmHost_0@@Base+0x6c270>
  fb1058:	48 8d 7d d8          	lea    -0x28(%rbp),%rdi
  fb105c:	e8 cf 5f 3b 00       	call   1367030 <VerifyCdmHost_0@@Base+0x3b5e90>
  fb1061:	48 89 df             	mov    %rbx,%rdi
  fb1064:	4c 89 fe             	mov    %r15,%rsi
  fb1067:	e8 04 93 01 00       	call   fca370 <VerifyCdmHost_0@@Base+0x191d0>
  fb106c:	49 89 c4             	mov    %rax,%r12
  fb106f:	bf 18 00 00 00       	mov    $0x18,%edi
  fb1074:	e8 d7 b2 3c 00       	call   137c350 <VerifyCdmHost_0@@Base+0x3cb1b0>
  fb1079:	49 89 c6             	mov    %rax,%r14
  fb107c:	48 89 c7             	mov    %rax,%rdi
  fb107f:	48 89 de             	mov    %rbx,%rsi
  fb1082:	4c 89 fa             	mov    %r15,%rdx
  fb1085:	4c 89 e1             	mov    %r12,%rcx
  fb1088:	e8 23 54 00 00       	call   fb64b0 <VerifyCdmHost_0@@Base+0x5310>
  fb108d:	e9 ed 00 00 00       	jmp    fb117f <CreateCdmInstance@@Base+0x1ef>
  fb1092:	bf 0b 00 00 00       	mov    $0xb,%edi
  fb1097:	4c 89 c6             	mov    %r8,%rsi
  fb109a:	ff d1                	call   *%rcx
  fb109c:	48 85 c0             	test   %rax,%rax
  fb109f:	0f 84 d7 00 00 00    	je     fb117c <CreateCdmInstance@@Base+0x1ec>
  fb10a5:	49 89 c6             	mov    %rax,%r14
  fb10a8:	bf 40 00 00 00       	mov    $0x40,%edi
  fb10ad:	e8 9e b2 3c 00       	call   137c350 <VerifyCdmHost_0@@Base+0x3cb1b0>
  fb10b2:	48 89 c3             	mov    %rax,%rbx
  fb10b5:	48 89 c7             	mov    %rax,%rdi
  fb10b8:	4c 89 f6             	mov    %r14,%rsi
  fb10bb:	e8 50 bd ff ff       	call   face10 <InitializeCdmModule_4@@Base-0x40c0>
  fb10c0:	48 b8 00 00 00 00 fc 	movabs $0xfffffffc00000000,%rax
  fb10c7:	ff ff ff 
  fb10ca:	48 21 d8             	and    %rbx,%rax
  fb10cd:	48 8d 0d 2c 0f 53 00 	lea    0x530f2c(%rip),%rcx        # 14e2000 <__tls_get_addr@plt+0x11830>
  fb10d4:	48 3b 41 08          	cmp    0x8(%rcx),%rax
  fb10d8:	75 08                	jne    fb10e2 <CreateCdmInstance@@Base+0x152>
  fb10da:	48 89 df             	mov    %rbx,%rdi
  fb10dd:	e8 8e 2d 3c 00       	call   1373e70 <VerifyCdmHost_0@@Base+0x3c2cd0>
  fb10e2:	bf 38 00 00 00       	mov    $0x38,%edi
  fb10e7:	e8 64 b2 3c 00       	call   137c350 <VerifyCdmHost_0@@Base+0x3cb1b0>
  fb10ec:	49 89 c6             	mov    %rax,%r14
  fb10ef:	48 8d 35 ba 00 00 00 	lea    0xba(%rip),%rsi        # fb11b0 <VerifyCdmHost_0@@Base+0x10>
  fb10f6:	48 8d 15 13 01 00 00 	lea    0x113(%rip),%rdx        # fb1210 <VerifyCdmHost_0@@Base+0x70>
  fb10fd:	48 89 c7             	mov    %rax,%rdi
  fb1100:	e8 7b 5e 3b 00       	call   1366f80 <VerifyCdmHost_0@@Base+0x3b5de0>
  fb1105:	49 c7 46 20 19 00 00 	movq   $0x19,0x20(%r14)
  fb110c:	00 
  fb110d:	49 c7 46 28 00 00 00 	movq   $0x0,0x28(%r14)
  fb1114:	00 
  fb1115:	49 89 5e 30          	mov    %rbx,0x30(%r14)
  fb1119:	bf 30 00 00 00       	mov    $0x30,%edi
  fb111e:	e8 2d b2 3c 00       	call   137c350 <VerifyCdmHost_0@@Base+0x3cb1b0>
  fb1123:	49 89 c7             	mov    %rax,%r15
  fb1126:	48 c7 45 d8 00 00 00 	movq   $0x0,-0x28(%rbp)
  fb112d:	00 
  fb112e:	48 89 c7             	mov    %rax,%rdi
  fb1131:	4c 89 f6             	mov    %r14,%rsi
  fb1134:	e8 d7 c2 06 00       	call   101d410 <VerifyCdmHost_0@@Base+0x6c270>
  fb1139:	48 8d 7d d8          	lea    -0x28(%rbp),%rdi
  fb113d:	e8 ee 5e 3b 00       	call   1367030 <VerifyCdmHost_0@@Base+0x3b5e90>
  fb1142:	48 89 df             	mov    %rbx,%rdi
  fb1145:	4c 89 fe             	mov    %r15,%rsi
  fb1148:	e8 23 92 01 00       	call   fca370 <VerifyCdmHost_0@@Base+0x191d0>
  fb114d:	49 89 c6             	mov    %rax,%r14
  fb1150:	bf 18 00 00 00       	mov    $0x18,%edi
  fb1155:	e8 f6 b1 3c 00       	call   137c350 <VerifyCdmHost_0@@Base+0x3cb1b0>
  fb115a:	49 89 c4             	mov    %rax,%r12
  fb115d:	48 89 c7             	mov    %rax,%rdi
  fb1160:	48 89 de             	mov    %rbx,%rsi
  fb1163:	4c 89 fa             	mov    %r15,%rdx
  fb1166:	4c 89 f1             	mov    %r14,%rcx
  fb1169:	e8 42 53 00 00       	call   fb64b0 <VerifyCdmHost_0@@Base+0x5310>
  fb116e:	4d 8d 74 24 08       	lea    0x8(%r12),%r14
  fb1173:	4d 85 e4             	test   %r12,%r12
  fb1176:	4d 0f 44 f4          	cmove  %r12,%r14
  fb117a:	eb 03                	jmp    fb117f <CreateCdmInstance@@Base+0x1ef>
  fb117c:	45 31 f6             	xor    %r14d,%r14d
  fb117f:	4c 89 f0             	mov    %r14,%rax
  fb1182:	48 83 c4 10          	add    $0x10,%rsp
  fb1186:	5b                   	pop    %rbx
  fb1187:	41 5c                	pop    %r12
  fb1189:	41 5e                	pop    %r14
  fb118b:	41 5f                	pop    %r15
  fb118d:	5d                   	pop    %rbp
  fb118e:	c3                   	ret
  fb118f:	cc                   	int3

0000000000fb1190 <GetCdmVersion@@Base>:
  fb1190:	55                   	push   %rbp
  fb1191:	48 89 e5             	mov    %rsp,%rbp
  fb1194:	48 8d 05 01 ad 07 ff 	lea    -0xf852ff(%rip),%rax        # 2be9c <InitializeCdmModule_4@@Base-0xf85034>
  fb119b:	5d                   	pop    %rbp
  fb119c:	c3                   	ret
  fb119d:	cc                   	int3
  fb119e:	cc                   	int3
  fb119f:	cc                   	int3

0000000000fb11a0 <VerifyCdmHost_0@@Base>:
  fb11a0:	55                   	push   %rbp
  fb11a1:	48 89 e5             	mov    %rsp,%rbp
  fb11a4:	b0 01                	mov    $0x1,%al
  fb11a6:	5d                   	pop    %rbp
  fb11a7:	c3                   	ret
  fb11a8:	cc                   	int3
  fb11a9:	cc                   	int3
  fb11aa:	cc                   	int3
  fb11ab:	cc                   	int3
  fb11ac:	cc                   	int3
  fb11ad:	cc                   	int3
  fb11ae:	cc                   	int3
  fb11af:	cc                   	int3
