import java.lang.reflect.*;
import java.util.*;
import java.io.*;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;

public class SharedAppearanceProbe {
    public static void main(String[] args) throws Exception {
        final byte[] image = Files.readAllBytes(Paths.get(args[0]));
        final int[] requests = {0};
        URL.setURLStreamHandlerFactory(new URLStreamHandlerFactory() {
            public URLStreamHandler createURLStreamHandler(String protocol) {
                if (!"https".equals(protocol)) return null;
                return new URLStreamHandler() {
                    protected URLConnection openConnection(URL url) throws IOException {
                        if (!"luxmc-r92.pages.dev".equals(url.getHost()) || !url.getPath().equals("/api/appearance/LuxmcPeer")) throw new IOException("Unexpected address");
                        return new HttpURLConnection(url) {
                            public void connect() {}
                            public void disconnect() {}
                            public boolean usingProxy() { return false; }
                            public int getResponseCode() { requests[0]++; return url.getQuery().equals("asset=skin") ? 200 : 404; }
                            public String getContentType() { return "image/png"; }
                            public long getContentLengthLong() { return image.length; }
                            public String getHeaderField(String name) { return name.equals("X-Luxmc-Model") ? "slim" : null; }
                            public InputStream getInputStream() { return new ByteArrayInputStream(image); }
                        };
                    }
                };
            }
        });
        Class<?> profile = Class.forName("com.mojang.authlib.GameProfile");
        Object peer = profile.getConstructor(UUID.class, String.class).newInstance(UUID.nameUUIDFromBytes("OfflinePlayer:LuxmcPeer".getBytes(StandardCharsets.UTF_8)), "LuxmcPeer");
        Class<?> bridge = Class.forName("io.github.luxmc.client.AppearanceAgent", true, null);
        Method textures = bridge.getMethod("localTextures", Object.class, String.class);
        Map<?, ?> result = (Map<?, ?>)textures.invoke(null, peer, "java/util/Map");
        if (result == null || result.size() != 1) throw new AssertionError("Shared skin missing");
        Object skin = result.values().iterator().next();
        String address = (String)skin.getClass().getMethod("getUrl").invoke(skin);
        ByteArrayOutputStream received = new ByteArrayOutputStream();
        try (InputStream input = new URL(address).openStream()) {
            byte[] buffer = new byte[4096]; int count;
            while ((count = input.read(buffer)) != -1) received.write(buffer, 0, count);
        }
        if (!Arrays.equals(image, received.toByteArray())) throw new AssertionError("Shared image differs");
        if (!"slim".equals(skin.getClass().getMethod("getMetadata", String.class).invoke(skin, "model"))) throw new AssertionError("Shared model differs");
        textures.invoke(null, peer, "java/util/Map");
        if (requests[0] != 2) throw new AssertionError("Textures are not cached");
        Object official = profile.getConstructor(UUID.class, String.class).newInstance(UUID.randomUUID(), "LuxmcPeer");
        if (textures.invoke(null, official, "java/util/Map") != null) throw new AssertionError("Official identity replaced");
        System.out.println("Shared Luxmc appearance verified; official identity preserved");
    }
}
