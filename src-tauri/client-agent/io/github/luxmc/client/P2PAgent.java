package io.github.luxmc.client;

import java.io.FileInputStream;
import java.lang.instrument.ClassFileTransformer;
import java.lang.instrument.Instrumentation;
import java.lang.reflect.Field;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;
import java.security.ProtectionDomain;
import java.util.HashMap;
import java.util.IdentityHashMap;
import java.util.Map;
import java.util.Properties;
import org.objectweb.asm.*;

public final class P2PAgent {
    private static volatile long checked;
    private static volatile long expires;
    private static volatile int port;

    public static void install(Instrumentation instrumentation) {
        instrumentation.addTransformer(new ClassFileTransformer() {
            public byte[] transform(ClassLoader loader, String name, Class<?> type, ProtectionDomain domain, byte[] bytes) {
                if (loader == null || name == null || name.startsWith("io/github/luxmc/") || name.startsWith("java/")) return null;
                String constants = new String(bytes, StandardCharsets.ISO_8859_1);
                if (!constants.contains("java/security/KeyPair") || !constants.contains("com/mojang/authlib/GameProfile")) return null;
                final Map<String, String> targets = new HashMap<String, String>();
                ClassReader reader = new ClassReader(bytes);
                reader.accept(new ClassVisitor(Opcodes.ASM9) {
                    public MethodVisitor visitMethod(int access, final String method, final String descriptor, String signature, String[] exceptions) {
                        if ((access & Opcodes.ACC_STATIC) != 0) return null;
                        return new MethodVisitor(Opcodes.ASM9) {
                            boolean encryption;
                            String authentication;
                            public void visitMethodInsn(int opcode, String owner, String name, String desc, boolean itf) {
                                if (authentication == null && owner.equals("net/minecraft/server/MinecraftServer") && desc.equals("()Z")) authentication = owner + "." + name;
                                if (owner.equals("java/security/KeyPair") && name.equals("getPublic")) encryption = true;
                            }
                            public void visitEnd() { if (encryption && authentication != null) targets.put(method + descriptor, authentication); }
                        };
                    }
                }, ClassReader.SKIP_DEBUG | ClassReader.SKIP_FRAMES);
                if (targets.isEmpty()) return null;
                ClassWriter writer = new ClassWriter(reader, ClassWriter.COMPUTE_MAXS);
                reader.accept(new ClassVisitor(Opcodes.ASM9, writer) {
                    public MethodVisitor visitMethod(int access, String method, String descriptor, String signature, String[] exceptions) {
                        MethodVisitor delegate = super.visitMethod(access, method, descriptor, signature, exceptions);
                        final String target = targets.get(method + descriptor);
                        if (target == null) return delegate;
                        return new MethodVisitor(Opcodes.ASM9, delegate) {
                            public void visitMethodInsn(int opcode, String owner, String name, String desc, boolean itf) {
                                super.visitMethodInsn(opcode, owner, name, desc, itf);
                                if (desc.equals("()Z") && target.equals(owner + "." + name)) {
                                    super.visitVarInsn(Opcodes.ALOAD, 0);
                                    super.visitMethodInsn(Opcodes.INVOKESTATIC, "io/github/luxmc/client/P2PAgent", "authenticate", "(ZLjava/lang/Object;)Z", false);
                                }
                            }
                        };
                    }
                }, 0);
                System.out.println("[LUXMC_P2P] Private LAN login support: " + name);
                return writer.toByteArray();
            }
        }, false);
    }

    public static boolean authenticate(boolean original, Object loginHandler) {
        if (!original || !active()) return original;
        return !loopbackPeer(loginHandler, 0, new IdentityHashMap<Object, Boolean>());
    }

    private static synchronized boolean active() {
        long now = System.currentTimeMillis();
        if (now - checked > 500) {
            checked = now; expires = 0; port = 0;
            String path = System.getProperty("luxmc.p2p.session", "");
            if (!path.isEmpty()) {
                Properties properties = new Properties();
                try (FileInputStream stream = new FileInputStream(path)) {
                    properties.load(stream);
                    expires = Long.parseLong(properties.getProperty("expires", "0"));
                    port = Integer.parseInt(properties.getProperty("port", "0"));
                } catch (Exception ignored) {}
            }
        }
        return port > 0 && port <= 65535 && expires > now && expires - now <= 10000;
    }

    private static boolean loopbackPeer(Object value, int depth, IdentityHashMap<Object, Boolean> visited) {
        if (value == null || depth > 3 || visited.size() >= 32 || visited.put(value, Boolean.TRUE) != null) return false;
        for (Class<?> type = value.getClass(); type != null; type = type.getSuperclass()) {
            if (type.getName().equals("net.minecraft.server.MinecraftServer")) return false;
        }
        if (value instanceof InetSocketAddress) {
            InetSocketAddress address = (InetSocketAddress) value;
            return address.getAddress() != null && address.getAddress().isLoopbackAddress();
        }
        try {
            Object remote = value.getClass().getMethod("remoteAddress").invoke(value);
            Object local = value.getClass().getMethod("localAddress").invoke(value);
            if (local instanceof InetSocketAddress && ((InetSocketAddress) local).getPort() == port && remote instanceof InetSocketAddress) {
                InetSocketAddress address = (InetSocketAddress) remote;
                return address.getAddress() != null && address.getAddress().isLoopbackAddress();
            }
        } catch (Exception ignored) {}
        for (Class<?> type = value.getClass(); type != null && type != Object.class; type = type.getSuperclass()) {
            for (Field field : type.getDeclaredFields()) {
                if (java.lang.reflect.Modifier.isStatic(field.getModifiers()) || field.getType().isPrimitive()) continue;
                try {
                    field.setAccessible(true);
                    Object child = field.get(value);
                    if (child != null && !(child instanceof InetSocketAddress) && !child.getClass().getName().startsWith("java.") && loopbackPeer(child, depth + 1, visited)) return true;
                } catch (Exception ignored) {}
            }
        }
        return false;
    }
}
