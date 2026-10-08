import java.lang.reflect.Field;
import java.lang.reflect.Method;
public final class GlfwInitializationProbe {
    private static int calls;
    public static Long beforeInitialization() { calls++; return 0L; }
    public static void main(String[] args) throws Exception {
        Class<?> agent = Class.forName("io.github.luxmc.client.LuxmcClientAgent");
        Field context = agent.getDeclaredField("glfwGetCurrentContextMethod");
        context.setAccessible(true);
        context.set(null, GlfwInitializationProbe.class.getMethod("beforeInitialization"));
        Method resolve = agent.getDeclaredMethod("resolveGlfwWindowHandle");
        resolve.setAccessible(true);
        resolve.invoke(null);
        if(calls != 0) throw new AssertionError("GLFW was called before a Minecraft window existed");
        System.out.println("PASS: no GLFW call before Minecraft window initialization");
    }
}
