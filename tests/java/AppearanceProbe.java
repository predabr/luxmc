import java.lang.reflect.*;
import java.util.*;
import java.io.*;
import java.net.*;
public class AppearanceProbe {
    static Object profile(String uuid) throws Exception {
        return Class.forName("com.mojang.authlib.GameProfile").getConstructor(UUID.class, String.class).newInstance(UUID.fromString(uuid), "Player");
    }
    public static void main(String[] args) throws Exception {
        Class<?> service;
        try { service = Class.forName("com.mojang.authlib.yggdrasil.YggdrasilMinecraftSessionService"); }
        catch (ClassNotFoundException ignored) { service = Class.forName("com.mojang.authlib.services.MinecraftServicesSessionService"); }
        Class<?> unsafeType = Class.forName("sun.misc.Unsafe");
        Field unsafeField = unsafeType.getDeclaredField("theUnsafe"); unsafeField.setAccessible(true);
        Object instance = unsafeType.getMethod("allocateInstance", Class.class).invoke(unsafeField.get(null), service);
        Object own = profile("01234567-89ab-cdef-0123-456789abcdef");
        Object other = profile("11234567-89ab-cdef-0123-456789abcdef");
        Class<?> bridge = Class.forName("io.github.luxmc.client.AppearanceAgent", true, null);
        if (bridge.getMethod("localTextures", Object.class, String.class).invoke(null, other, "java/util/Map") != null) throw new AssertionError("Other player changed");
        Object skin;
        Object cape;
        try {
            Object result = service.getMethod("getTextures", own.getClass(), boolean.class).invoke(instance, own, true);
            Map<?,?> textures = (Map<?,?>)result;
            skin = null; cape = null;
            for (Map.Entry<?,?> entry : textures.entrySet()) {
                if (entry.getKey().toString().equals("SKIN")) skin = entry.getValue();
                if (entry.getKey().toString().equals("CAPE")) cape = entry.getValue();
            }
        } catch (NoSuchMethodException modern) {
            Object property = service.getMethod("getPackedTextures", own.getClass()).invoke(instance, own);
            Object result = service.getMethod("unpackTextures", property.getClass()).invoke(instance, property);
            skin = result.getClass().getMethod("skin").invoke(result);
            cape = result.getClass().getMethod("cape").invoke(result);
        }
        if (skin == null || (cape != null) != !System.getProperty("luxmc.appearance.cape", "").isEmpty()) throw new AssertionError("Missing local appearance");
        if (!skin.getClass().getMethod("getMetadata", String.class).invoke(skin, "model").equals(System.getProperty("luxmc.appearance.model"))) throw new AssertionError("Wrong model");
        String url = (String)skin.getClass().getMethod("getUrl").invoke(skin);
        URL address = new URL(url);
        if (!address.getHost().equals("127.0.0.1")) throw new AssertionError("Not local");
        ByteArrayOutputStream output = new ByteArrayOutputStream();
        try (InputStream input = address.openStream()) { byte[] buffer = new byte[4096]; int n; while ((n = input.read(buffer)) != -1) output.write(buffer,0,n); }
        byte[] expected = java.nio.file.Files.readAllBytes(java.nio.file.Paths.get(System.getProperty("luxmc.appearance.skin")));
        if (!Arrays.equals(expected,output.toByteArray())) throw new AssertionError("Texture bytes differ");
        System.out.println("Appearance verified: " + service.getName());
    }
}
