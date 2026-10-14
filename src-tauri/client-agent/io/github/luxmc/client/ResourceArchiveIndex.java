package io.github.luxmc.client;

import java.lang.instrument.*;
import java.security.ProtectionDomain;
import java.util.*;
import java.util.zip.*;
import org.objectweb.asm.*;

public final class ResourceArchiveIndex {
    private static final int MAX_ENTRIES = 50000;
    private static final long MAX_BYTES = 16L * 1024 * 1024;
    private static final Map<ZipFile, Index> cache = new WeakHashMap<ZipFile, Index>();
    private static final ClassFileHints.Hint ZIP = new ClassFileHints.Hint("java/util/zip/ZipFile");
    private static final class Index {
        final List<ZipEntry> entries;
        final long bytes;
        Index(List<ZipEntry> entries, long bytes) { this.entries = entries; this.bytes = bytes; }
    }
    public static void install(Instrumentation instrumentation) {
        if (!"true".equals(System.getProperty("luxmc.resourceIndex"))) return;
        instrumentation.addTransformer(new ClassFileTransformer() {
            public byte[] transform(ClassLoader loader, String name, Class<?> type, ProtectionDomain domain, byte[] bytes) {
                if (loader == null || name == null || !(name.startsWith("net/minecraft/") || name.indexOf('/') < 0 && name.length() <= 8 && !name.isEmpty() && name.charAt(0) >= 'a' && name.charAt(0) <= 'z') || !ClassFileHints.contains(bytes, ZIP)) return null;
                try {
                    ClassReader reader = new ClassReader(bytes);
                    ClassWriter writer = new ClassWriter(reader, 0);
                    final boolean[] modified = new boolean[1];
                    reader.accept(new ClassVisitor(Opcodes.ASM9, writer) {
                        public MethodVisitor visitMethod(int access, String method, String descriptor, String signature, String[] exceptions) {
                            return new MethodVisitor(Opcodes.ASM9, super.visitMethod(access, method, descriptor, signature, exceptions)) {
                                public void visitMethodInsn(int opcode, String owner, String name, String descriptor, boolean itf) {
                                    if (opcode == Opcodes.INVOKEVIRTUAL && owner.equals("java/util/zip/ZipFile") && name.equals("entries") && descriptor.equals("()Ljava/util/Enumeration;")) {
                                        super.visitMethodInsn(Opcodes.INVOKESTATIC, "io/github/luxmc/client/ResourceArchiveIndex", "entries", "(Ljava/util/zip/ZipFile;)Ljava/util/Enumeration;", false);
                                        modified[0] = true;
                                    } else super.visitMethodInsn(opcode, owner, name, descriptor, itf);
                                }
                            };
                        }
                    }, 0);
                    if (!modified[0]) return null;
                    System.out.println("[LUXMC_CLIENT] Resource archive index: " + name);
                    return writer.toByteArray();
                } catch (Throwable error) { System.err.println("[LUXMC_CLIENT] Resource index unavailable: " + error.getClass().getSimpleName()); return null; }
            }
        });
    }
    public static Enumeration<? extends ZipEntry> entries(final ZipFile zip) {
        final Enumeration<? extends ZipEntry> original = zip.entries();
        if (zip.getClass() != ZipFile.class || zip.size() > MAX_ENTRIES) return original;
        Index found;
        synchronized (cache) {
            found = cache.get(zip);
            if (found == null) {
                List<ZipEntry> entries = new ArrayList<ZipEntry>(zip.size());
                long weight = 0;
                while (original.hasMoreElements()) {
                    ZipEntry entry = original.nextElement();
                    weight += 192L + entry.getName().length() * 2L + (entry.getComment() == null ? 0 : entry.getComment().length() * 2L) + (entry.getExtra() == null ? 0 : entry.getExtra().length);
                    if (weight > MAX_BYTES) return zip.entries();
                    entries.add((ZipEntry)entry.clone());
                }
                long used = 0;
                for (Index index : cache.values()) used += index.bytes;
                Iterator<Index> old = cache.values().iterator();
                while (used + weight > MAX_BYTES && old.hasNext()) { used -= old.next().bytes; old.remove(); }
                found = new Index(Collections.unmodifiableList(entries), weight);
                cache.put(zip, found);
            }
        }
        final List<ZipEntry> indexed = found.entries;
        final Enumeration<? extends ZipEntry> state = zip.entries();
        return new Enumeration<ZipEntry>() {
            private int position;
            public boolean hasMoreElements() { boolean open = state.hasMoreElements(); return position < indexed.size() && open; }
            public ZipEntry nextElement() {
                synchronized (zip) {
                    zip.size();
                    if (position >= indexed.size()) throw new NoSuchElementException();
                    return (ZipEntry)indexed.get(position++).clone();
                }
            }
        };
    }
}
