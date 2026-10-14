import java.io.*;
import java.lang.instrument.*;
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;
import java.util.zip.*;
import org.objectweb.asm.*;

public class ResourceArchiveProbe {
    static Enumeration<? extends ZipEntry> cached(Method method, ZipFile zip) throws Exception {
        return (Enumeration<? extends ZipEntry>)method.invoke(null, zip);
    }
    static long enumerate(Enumeration<? extends ZipEntry> entries) {
        long checksum = 0;
        while (entries.hasMoreElements()) { ZipEntry entry = entries.nextElement(); checksum += entry.getName().hashCode() + entry.getSize() + entry.getCrc(); }
        return checksum;
    }
    public static void main(String[] args) throws Exception {
        Class<?> index = Class.forName("io.github.luxmc.client.ResourceArchiveIndex", true, null);
        Method entries = index.getMethod("entries", ZipFile.class);
        Path path = Files.createTempFile("luxmc-resource-index-", ".zip");
        try {
            try (ZipOutputStream out = new ZipOutputStream(Files.newOutputStream(path))) {
                for (int i = 0; i < 12000; i++) {
                    ZipEntry entry = new ZipEntry("assets/probe/textures/árvore_" + i + ".png");
                    entry.setComment("original"); out.putNextEntry(entry); out.write(new byte[] {1, 2, 3}); out.closeEntry();
                }
            }
            try (ZipFile zip = new ZipFile(path.toFile())) {
                long checksum = enumerate(zip.entries());
                if (enumerate(cached(entries, zip)) != checksum) throw new AssertionError("Entries changed");
                ZipEntry changed = cached(entries, zip).nextElement(); changed.setComment("changed"); changed.setSize(999);
                ZipEntry again = cached(entries, zip).nextElement();
                if (!"original".equals(again.getComment()) || again.getSize() != 3) throw new AssertionError("Mutation leaked into cache");
                try (InputStream in = zip.getInputStream(again)) { if (in.read() != 1) throw new AssertionError("Resource bytes changed"); }
                long started = System.nanoTime();
                for (int i = 0; i < 80; i++) if (enumerate(zip.entries()) != checksum) throw new AssertionError();
                long baseline = System.nanoTime() - started;
                started = System.nanoTime();
                for (int i = 0; i < 80; i++) if (enumerate(cached(entries, zip)) != checksum) throw new AssertionError();
                long indexed = System.nanoTime() - started;
                System.out.println("Resource enumeration baselineMs=" + baseline / 1000000 + " indexedMs=" + indexed / 1000000);
            }
            ZipFile closed = new ZipFile(path.toFile()); Enumeration<? extends ZipEntry> old = cached(entries, closed); closed.close();
            try { old.nextElement(); throw new AssertionError("Closed archive accepted"); } catch (IllegalStateException expected) {}
            try (java.util.jar.JarFile jar = new java.util.jar.JarFile(path.toFile())) {
                if (!(cached(entries, jar).nextElement() instanceof java.util.jar.JarEntry)) throw new AssertionError("JarFile enumeration changed");
            }
            final List<ClassFileTransformer> transformers = new ArrayList<ClassFileTransformer>();
            Instrumentation mock = (Instrumentation)Proxy.newProxyInstance(ResourceArchiveProbe.class.getClassLoader(), new Class<?>[]{Instrumentation.class}, (proxy, method, values) -> { if (method.getName().equals("addTransformer")) transformers.add((ClassFileTransformer)values[0]); return null; });
            System.clearProperty("luxmc.resourceIndex");
            index.getMethod("install", Instrumentation.class).invoke(null, mock);
            if (!transformers.isEmpty()) throw new AssertionError("Experimental hook enabled by default");
            System.setProperty("luxmc.resourceIndex", "true");
            index.getMethod("install", Instrumentation.class).invoke(null, mock);
            ClassWriter writer = new ClassWriter(0);
            writer.visit(Opcodes.V1_8, Opcodes.ACC_PUBLIC, "net/minecraft/fixture/ArchiveReader", null, "java/lang/Object", null);
            MethodVisitor method = writer.visitMethod(Opcodes.ACC_PUBLIC | Opcodes.ACC_STATIC, "entries", "(Ljava/util/zip/ZipFile;)Ljava/util/Enumeration;", null, null);
            method.visitCode(); method.visitVarInsn(Opcodes.ALOAD, 0); method.visitMethodInsn(Opcodes.INVOKEVIRTUAL, "java/util/zip/ZipFile", "entries", "()Ljava/util/Enumeration;", false); method.visitInsn(Opcodes.ARETURN); method.visitMaxs(1, 1); method.visitEnd(); writer.visitEnd();
            byte[] transformed = transformers.get(0).transform(ResourceArchiveProbe.class.getClassLoader(), "net/minecraft/fixture/ArchiveReader", null, null, writer.toByteArray());
            if (transformed == null) throw new AssertionError("Resource hook missing");
            Class<?> reader = new ClassLoader(ResourceArchiveProbe.class.getClassLoader()) { Class<?> load(byte[] bytes) { return defineClass(null, bytes, 0, bytes.length); } }.load(transformed);
            try (ZipFile zip = new ZipFile(path.toFile())) {
                if (enumerate((Enumeration<? extends ZipEntry>)reader.getMethod("entries", ZipFile.class).invoke(null, zip)) != enumerate(zip.entries())) throw new AssertionError("Transformed enumeration changed");
            }
            if (transformers.get(0).transform(ResourceArchiveProbe.class.getClassLoader(), "example/Other", null, null, writer.toByteArray()) != null) throw new AssertionError("Third party class changed");
            System.out.println("PASS resource order, contents, metadata, mutation isolation, close semantics and transformed bytecode");
        } finally { Files.deleteIfExists(path); }
    }
}
