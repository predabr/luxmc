import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.lang.management.ManagementFactory;
import java.util.*;
import java.util.zip.*;
import io.github.luxmc.client.ClassFileHints;

public class ClassScanBenchmark {
    static final String[] MARKERS = {"Started serving on", "Published LAN server on port", "Started on ", "java/security/KeyPair", "com/mojang/authlib/GameProfile"};
    static final ClassFileHints.Hint[] NEEDLES = Arrays.stream(MARKERS).map(ClassFileHints.Hint::new).toArray(ClassFileHints.Hint[]::new);
    static int scan(List<byte[]> classes, boolean bounded) {
        int matched = 0;
        for (byte[] bytes : classes) {
            if (bounded) {
                if (ClassFileHints.contains(bytes, NEEDLES[0]) || ClassFileHints.contains(bytes, NEEDLES[1]) || ClassFileHints.contains(bytes, NEEDLES[2])) matched++;
                if (ClassFileHints.contains(bytes, NEEDLES[3]) && ClassFileHints.contains(bytes, NEEDLES[4])) matched++;
            } else {
                String lan = new String(bytes, StandardCharsets.ISO_8859_1);
                if (lan.contains(MARKERS[0]) || lan.contains(MARKERS[1]) || lan.contains(MARKERS[2])) matched++;
                String p2p = new String(bytes, StandardCharsets.ISO_8859_1);
                if (p2p.contains(MARKERS[3]) && p2p.contains(MARKERS[4])) matched++;
            }
        }
        return matched;
    }
    public static void main(String[] args) throws Exception {
        List<byte[]> classes = new ArrayList<byte[]>();
        try (ZipFile archive = new ZipFile(args[0])) {
            Enumeration<? extends ZipEntry> entries = archive.entries();
            while (entries.hasMoreElements() && classes.size() < 3000) {
                ZipEntry entry = entries.nextElement();
                if (entry.getName().endsWith(".class")) {
                    try (java.io.InputStream input = archive.getInputStream(entry)) { classes.add(input.readAllBytes()); }
                }
            }
        }
        com.sun.management.ThreadMXBean bean = (com.sun.management.ThreadMXBean)ManagementFactory.getThreadMXBean();
        bean.setThreadAllocatedMemoryEnabled(true);
        int expected = scan(classes, false);
        if (scan(classes, true) != expected) throw new AssertionError("Marker selection changed");
        for (int round = 0; round < 4; round++) { scan(classes, false); scan(classes, true); }
        for (boolean bounded : new boolean[] {false, true}) {
            long before = bean.getThreadAllocatedBytes(Thread.currentThread().getId());
            long started = System.nanoTime();
            for (int round = 0; round < 5; round++) if (scan(classes, bounded) != expected) throw new AssertionError("Unstable selection");
            double seconds = (System.nanoTime() - started) / 1e9;
            long bytes = bean.getThreadAllocatedBytes(Thread.currentThread().getId()) - before;
            System.out.println("{\"bounded\":" + bounded + ",\"classes\":" + classes.size() + ",\"rounds\":5,\"matches\":" + expected + ",\"allocatedBytes\":" + bytes + ",\"seconds\":" + seconds + "}");
        }
    }
}
