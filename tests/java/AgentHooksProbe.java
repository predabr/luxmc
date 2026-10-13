import java.lang.instrument.*;
import java.lang.reflect.*;
import java.nio.charset.StandardCharsets;
import java.util.*;
import org.objectweb.asm.*;

public class AgentHooksProbe {
    static byte[] fixture(String message, boolean login) {
        ClassWriter writer = new ClassWriter(0);
        writer.visit(Opcodes.V1_8, Opcodes.ACC_PUBLIC, "fixture/IntegratedServer", null, "net/minecraft/server/MinecraftServer", null);
        writer.newConst(1L);
        writer.newConst(2.0);
        MethodVisitor method = writer.visitMethod(Opcodes.ACC_PUBLIC, "publish", "()Z", null, null);
        method.visitCode();
        method.visitLdcInsn(message); method.visitInsn(Opcodes.POP);
        if (login) {
            method.visitLdcInsn(org.objectweb.asm.Type.getObjectType("com/mojang/authlib/GameProfile")); method.visitInsn(Opcodes.POP);
            method.visitInsn(Opcodes.ACONST_NULL);
            method.visitMethodInsn(Opcodes.INVOKEVIRTUAL, "java/security/KeyPair", "getPublic", "()Ljava/security/PublicKey;", false);
            method.visitInsn(Opcodes.POP);
            method.visitVarInsn(Opcodes.ALOAD, 0);
            method.visitMethodInsn(Opcodes.INVOKEVIRTUAL, "net/minecraft/server/MinecraftServer", "isOnlineMode", "()Z", false);
        } else {
            method.visitInsn(Opcodes.ACONST_NULL); method.visitInsn(Opcodes.ACONST_NULL); method.visitIntInsn(Opcodes.SIPUSH, 25565);
            method.visitMethodInsn(Opcodes.INVOKEVIRTUAL, "fixture/Connection", "bind", "(Ljava/net/InetAddress;I)V", false);
            method.visitInsn(Opcodes.ICONST_1);
        }
        method.visitInsn(Opcodes.IRETURN); method.visitMaxs(3, 1); method.visitEnd(); writer.visitEnd();
        return writer.toByteArray();
    }

    public static void main(String[] args) throws Exception {
        final List<ClassFileTransformer> transformers = new ArrayList<ClassFileTransformer>();
        Instrumentation instrumentation = (Instrumentation)Proxy.newProxyInstance(AgentHooksProbe.class.getClassLoader(), new Class<?>[] {Instrumentation.class}, new InvocationHandler() {
            public Object invoke(Object proxy, Method method, Object[] arguments) {
                if (method.getName().equals("addTransformer")) transformers.add((ClassFileTransformer)arguments[0]);
                return null;
            }
        });
        Class<?> lan = Class.forName("io.github.luxmc.client.LanAgent", true, null);
        Class<?> p2p = Class.forName("io.github.luxmc.client.P2PAgent", true, null);
        lan.getMethod("install", Instrumentation.class).invoke(null, instrumentation);
        p2p.getMethod("install", Instrumentation.class).invoke(null, instrumentation);
        Class<?> hints = Class.forName("io.github.luxmc.client.ClassFileHints", true, null);
        Method contains = hints.getMethod("contains", byte[].class, byte[].class);
        for (String message : new String[] {"Started serving on {}", "Published LAN server on port {}", "Started on {}"}) {
            byte[] source = fixture(message, false);
            byte[] result = transformers.get(0).transform(AgentHooksProbe.class.getClassLoader(), "fixture/IntegratedServer", null, null, source);
            if (result == null || !new String(result, StandardCharsets.ISO_8859_1).contains("opened") || !new String(result, StandardCharsets.ISO_8859_1).contains("prepare")) throw new AssertionError("LAN hook lost: " + message);
            if (!Boolean.TRUE.equals(contains.invoke(null, source, message.getBytes(StandardCharsets.US_ASCII)))) throw new AssertionError("Constant hint missing");
            for (int length : new int[] {0, 9, 12}) {
                if (Boolean.TRUE.equals(contains.invoke(null, Arrays.copyOf(source, length), message.getBytes(StandardCharsets.US_ASCII)))) throw new AssertionError("Truncated class accepted");
            }
        }
        byte[] unrelated = fixture("unrelated", false);
        if (transformers.get(0).transform(AgentHooksProbe.class.getClassLoader(), "fixture/Unrelated", null, null, unrelated) != null) throw new AssertionError("Unrelated class transformed");
        byte[] login = transformers.get(1).transform(AgentHooksProbe.class.getClassLoader(), "fixture/Login", null, null, fixture("login", true));
        if (login == null || !new String(login, StandardCharsets.ISO_8859_1).contains("authenticate")) throw new AssertionError("Private LAN login hook lost");
        System.out.println("LAN discovery and private login transformations preserved; bounded constant scanning verified");
    }
}
