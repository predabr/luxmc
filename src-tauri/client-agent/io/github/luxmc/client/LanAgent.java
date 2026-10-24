package io.github.luxmc.client;

import java.lang.instrument.ClassFileTransformer;
import java.lang.instrument.Instrumentation;
import java.lang.management.ManagementFactory;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.nio.file.StandardCopyOption;
import java.security.ProtectionDomain;
import java.util.HashSet;
import java.util.Set;
import org.objectweb.asm.*;

public final class LanAgent {
    private static final ThreadLocal<Integer> pending = new ThreadLocal<Integer>();
    private static final ClassFileHints.Hint SERVING = new ClassFileHints.Hint("Started serving on");
    private static final ClassFileHints.Hint PUBLISHED = new ClassFileHints.Hint("Published LAN server on port");
    private static final ClassFileHints.Hint STARTED = new ClassFileHints.Hint("Started on ");

    public static int capacity(int original) {
        String path = System.getProperty("luxmc.p2p.session", "");
        if (path.isEmpty()) return original;
        try { if (Files.size(Paths.get(path)) > 512) return original; } catch (Exception ignored) { return original; }
        try (java.io.InputStream stream = Files.newInputStream(Paths.get(path))) {
            java.util.Properties properties = new java.util.Properties();
            properties.load(stream);
            long remaining = Long.parseLong(properties.getProperty("expires", "0")) - System.currentTimeMillis();
            int port = Integer.parseInt(properties.getProperty("port", "0"));
            int capacity = Integer.parseInt(properties.getProperty("maxPlayers", "0"));
            if (remaining > 0 && remaining <= 10000 && port > 0 && port <= 65535 && (capacity == 10 || capacity == 20)) return Math.max(original, capacity);
        } catch (Exception ignored) {}
        return original;
    }

    public static void install(Instrumentation instrumentation) {
        instrumentation.addTransformer(new ClassFileTransformer() {
            public byte[] transform(ClassLoader loader, String name, Class<?> type, ProtectionDomain domain, byte[] bytes) {
                if (loader == null || !("net/minecraft/server/players/PlayerList".equals(name) || "net/minecraft/server/management/PlayerList".equals(name) || "net/minecraft/class_3324".equals(name))) return null;
                final Set<String> limits = new HashSet<String>();
                ClassReader reader = new ClassReader(bytes);
                reader.accept(new ClassVisitor(Opcodes.ASM9) {
                    public MethodVisitor visitMethod(int access, String method, String descriptor, String signature, String[] exceptions) {
                        if (!method.equals("<init>") || !descriptor.endsWith("I)V")) return null;
                        Type[] arguments = Type.getArgumentTypes(descriptor);
                        int slot = 1;
                        for (int index = 0; index < arguments.length - 1; index++) slot += arguments[index].getSize();
                        final int limitSlot = slot;
                        return new MethodVisitor(Opcodes.ASM9) {
                            boolean limit;
                            public void visitVarInsn(int opcode, int variable) { limit = opcode == Opcodes.ILOAD && variable == limitSlot; }
                            public void visitFieldInsn(int opcode, String owner, String field, String descriptor) {
                                if (limit && opcode == Opcodes.PUTFIELD && owner.equals(name) && descriptor.equals("I")) limits.add(field);
                                limit = false;
                            }
                            public void visitInsn(int opcode) { limit = false; }
                            public void visitMethodInsn(int opcode, String owner, String method, String descriptor, boolean itf) { limit = false; }
                        };
                    }
                }, ClassReader.SKIP_DEBUG | ClassReader.SKIP_FRAMES);
                if (limits.size() != 1) return null;
                ClassWriter writer = new ClassWriter(reader, ClassWriter.COMPUTE_MAXS);
                reader.accept(new ClassVisitor(Opcodes.ASM9, writer) {
                    public MethodVisitor visitMethod(int access, String method, String descriptor, String signature, String[] exceptions) {
                        return new MethodVisitor(Opcodes.ASM9, super.visitMethod(access, method, descriptor, signature, exceptions)) {
                            public void visitFieldInsn(int opcode, String owner, String field, String descriptor) {
                                super.visitFieldInsn(opcode, owner, field, descriptor);
                                if (opcode == Opcodes.GETFIELD && owner.equals(name) && descriptor.equals("I") && limits.contains(field)) super.visitMethodInsn(Opcodes.INVOKESTATIC, "io/github/luxmc/client/LanAgent", "capacity", "(I)I", false);
                            }
                        };
                    }
                }, 0);
                return writer.toByteArray();
            }
        }, false);
        instrumentation.addTransformer(new ClassFileTransformer() {
            public byte[] transform(ClassLoader loader, String name, Class<?> type, ProtectionDomain domain, byte[] bytes) {
                if (loader == null || name == null || name.startsWith("io/github/luxmc/") || name.startsWith("java/")) return null;
                if (!ClassFileHints.contains(bytes, SERVING) && !ClassFileHints.contains(bytes, PUBLISHED) && !ClassFileHints.contains(bytes, STARTED)) return null;
                final Set<String> targets = new HashSet<String>();
                ClassReader reader = new ClassReader(bytes);
                if (!reader.getSuperName().equals("net/minecraft/server/MinecraftServer")) return null;
                reader.accept(new ClassVisitor(Opcodes.ASM9) {
                    public MethodVisitor visitMethod(int access, final String method, final String descriptor, String signature, String[] exceptions) {
                        if ((access & Opcodes.ACC_STATIC) != 0) return null;
                        return new MethodVisitor(Opcodes.ASM9) {
                            boolean binding;
                            boolean publishing;
                            public void visitLdcInsn(Object value) {
                                if (value instanceof String && (((String) value).contains("Started serving on") || ((String) value).contains("Published LAN server on port") || ((String) value).contains("Started on "))) publishing = true;
                            }
                            public void visitMethodInsn(int opcode, String owner, String name, String desc, boolean itf) {
                                if (desc.equals("(Ljava/net/InetAddress;I)V") && opcode != Opcodes.INVOKESTATIC) binding = true;
                            }
                            public void visitEnd() { if (binding && publishing) targets.add(method + descriptor); }
                        };
                    }
                }, ClassReader.SKIP_DEBUG | ClassReader.SKIP_FRAMES);
                if (targets.isEmpty()) return null;
                ClassWriter writer = new ClassWriter(reader, ClassWriter.COMPUTE_MAXS);
                reader.accept(new ClassVisitor(Opcodes.ASM9, writer) {
                    public MethodVisitor visitMethod(int access, String method, String descriptor, String signature, String[] exceptions) {
                        MethodVisitor delegate = super.visitMethod(access, method, descriptor, signature, exceptions);
                        if (!targets.contains(method + descriptor)) return delegate;
                        return new MethodVisitor(Opcodes.ASM9, delegate) {
                            public void visitMethodInsn(int opcode, String owner, String name, String desc, boolean itf) {
                                boolean binding = desc.equals("(Ljava/net/InetAddress;I)V") && opcode != Opcodes.INVOKESTATIC;
                                if (binding) {
                                    super.visitInsn(Opcodes.DUP2);
                                    super.visitMethodInsn(Opcodes.INVOKESTATIC, "io/github/luxmc/client/LanAgent", "prepare", "(Ljava/net/InetAddress;I)V", false);
                                }
                                super.visitMethodInsn(opcode, owner, name, desc, itf);
                                if (binding) super.visitMethodInsn(Opcodes.INVOKESTATIC, "io/github/luxmc/client/LanAgent", "opened", "()V", false);
                            }
                        };
                    }
                }, 0);
                System.out.println("[LUXMC_LAN] Integrated world discovery: " + name);
                return writer.toByteArray();
            }
        }, false);
    }

    public static void prepare(java.net.InetAddress address, int port) { pending.set(port); }

    public static void opened() {
        Integer port = pending.get();
        pending.remove();
        String destination = System.getProperty("luxmc.lan.published", "");
        if (port == null || port <= 0 || port > 65535 || destination.isEmpty()) return;
        try {
            String pid = ManagementFactory.getRuntimeMXBean().getName().split("@", 2)[0];
            Long.parseLong(pid);
            Path path = Paths.get(destination);
            if (path.getParent() == null) return;
            Files.createDirectories(path.getParent());
            Path temporary = path.resolveSibling(path.getFileName() + ".tmp");
            Files.write(temporary, ("pid=" + pid + "\nport=" + port + "\n").getBytes(StandardCharsets.UTF_8));
            Files.move(temporary, path, StandardCopyOption.REPLACE_EXISTING);
            System.out.println("[LUXMC_LAN] Opened world on port " + port);
        } catch (Exception error) { System.err.println("[LUXMC_LAN] Discovery notice: " + error.getClass().getSimpleName()); }
    }
}
