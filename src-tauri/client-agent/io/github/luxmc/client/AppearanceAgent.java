package io.github.luxmc.client;

import java.io.*;
import java.lang.instrument.*;
import java.lang.reflect.*;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.security.*;
import java.util.*;
import java.util.concurrent.*;
import java.util.regex.*;
import org.objectweb.asm.*;

public final class AppearanceAgent {
    private static String owner;
    private static String skinUrl;
    private static String capeUrl;
    private static String model;
    private static final Map<String, byte[]> images = new ConcurrentHashMap<String, byte[]>();
    private static String textureBase;
    private static final Map<String, RemoteAppearance> remote = new ConcurrentHashMap<String, RemoteAppearance>();
    private static final Pattern PROFILE_NAME = Pattern.compile("\"profileName\"\\s*:\\s*\"([A-Za-z0-9_]{3,16})\"");
    private static final class RemoteAppearance {
        final String skin, cape, model;
        final long expires;
        RemoteAppearance(String skin, String cape, String model, long expires) { this.skin = skin; this.cape = cape; this.model = model; this.expires = expires; }
    }
    private static final Pattern PROFILE_ID = Pattern.compile("\"profileId\"\\s*:\\s*\"([a-fA-F0-9-]{32,36})\"");

    public static void install(Instrumentation instrumentation) {
        owner = System.getProperty("luxmc.appearance.uuid", "").replace("-", "").toLowerCase(Locale.ROOT);
        if (!owner.matches("[a-f0-9]{32}")) return;
        try {
            String skin = addImage(System.getProperty("luxmc.appearance.skin", ""));
            String cape = addImage(System.getProperty("luxmc.appearance.cape", ""));
            model = "slim".equals(System.getProperty("luxmc.appearance.model")) ? "slim" : "default";
            final ServerSocket server = new ServerSocket(0, 8, InetAddress.getByName("127.0.0.1"));
            String base = "http://127.0.0.1:" + server.getLocalPort();
            textureBase = base;
            skinUrl = skin == null ? null : base + skin;
            capeUrl = cape == null ? null : base + cape;
            final ThreadFactory factory = new ThreadFactory() {
                public Thread newThread(Runnable runnable) { Thread thread = new Thread(runnable, "Luxmc-Texture"); thread.setDaemon(true); return thread; }
            };
            final ThreadPoolExecutor workers = new ThreadPoolExecutor(2, 2, 0, TimeUnit.SECONDS, new ArrayBlockingQueue<Runnable>(8), factory);
            factory.newThread(new Runnable() {
                public void run() {
                    while (!server.isClosed()) {
                        try {
                            final Socket socket = server.accept();
                            try { workers.execute(new Runnable() { public void run() { serve(socket); } }); }
                            catch (RejectedExecutionException ignored) { socket.close(); }
                        } catch (IOException ignored) { break; }
                    }
                }
            }).start();
            instrumentation.addTransformer(new ClassFileTransformer() {
                public byte[] transform(ClassLoader loader, String name, Class<?> changed, ProtectionDomain domain, byte[] bytes) {
                    if (!"com/mojang/authlib/yggdrasil/YggdrasilMinecraftSessionService".equals(name) && !"com/mojang/authlib/services/MinecraftServicesSessionService".equals(name)) return null;
                    try {
                        ClassReader reader = new ClassReader(bytes);
                        ClassWriter writer = new ClassWriter(reader, ClassWriter.COMPUTE_MAXS);
                        reader.accept(new ClassVisitor(Opcodes.ASM9, writer) {
                            public MethodVisitor visitMethod(int access, String name, String descriptor, String signature, String[] exceptions) {
                                MethodVisitor original = super.visitMethod(access, name, descriptor, signature, exceptions);
                                org.objectweb.asm.Type[] args = org.objectweb.asm.Type.getArgumentTypes(descriptor);
                                if ((access & Opcodes.ACC_STATIC) != 0 || args.length == 0 || !("getTextures".equals(name) || "getPackedTextures".equals(name) || "unpackTextures".equals(name))) return original;
                                String input = args[0].getInternalName();
                                if (!"com/mojang/authlib/GameProfile".equals(input) && !"com/mojang/authlib/properties/Property".equals(input)) return original;
                                final String result = org.objectweb.asm.Type.getReturnType(descriptor).getInternalName();
                                return new MethodVisitor(Opcodes.ASM9, original) {
                                    public void visitCode() {
                                        super.visitCode();
                                        visitVarInsn(Opcodes.ALOAD, 1);
                                        visitLdcInsn(result);
                                        visitMethodInsn(Opcodes.INVOKESTATIC, "io/github/luxmc/client/AppearanceAgent", "localTextures", "(Ljava/lang/Object;Ljava/lang/String;)Ljava/lang/Object;", false);
                                        visitInsn(Opcodes.DUP);
                                        Label unchanged = new Label();
                                        visitJumpInsn(Opcodes.IFNULL, unchanged);
                                        visitTypeInsn(Opcodes.CHECKCAST, result);
                                        visitInsn(Opcodes.ARETURN);
                                        visitLabel(unchanged);
                                        visitFrame(Opcodes.F_SAME1, 0, null, 1, new Object[] {"java/lang/Object"});
                                        visitInsn(Opcodes.POP);
                                    }
                                };
                            }
                        }, 0);
                        System.out.println("[LUXMC_CLIENT] Local appearance adapter: " + name);
                        return writer.toByteArray();
                    } catch (Throwable error) { System.err.println("[LUXMC_CLIENT] Appearance adapter unavailable: " + error.getClass().getSimpleName()); return null; }
                }
            });
        } catch (Throwable error) { System.err.println("[LUXMC_CLIENT] Local appearance initialization failed: " + error.getClass().getSimpleName()); }
    }

    private static String addImage(String path) throws Exception {
        if (path.isEmpty()) return null;
        Path file = Paths.get(path);
        if (Files.size(file) > 3 * 1024 * 1024) throw new IOException("Texture too large");
        byte[] bytes = Files.readAllBytes(file);
        if (bytes.length < 24 || bytes[0] != (byte)137 || bytes[1] != 80 || bytes[2] != 78 || bytes[3] != 71) throw new IOException("Invalid PNG");
        return addBytes(bytes);
    }

    private static String addBytes(byte[] bytes) throws Exception {
        if (images.size() >= 128) throw new IOException("Texture cache full");
        java.awt.image.BufferedImage decoded = javax.imageio.ImageIO.read(new ByteArrayInputStream(bytes));
        if (decoded == null || decoded.getWidth() > 512 || decoded.getHeight() > 512) throw new IOException("Invalid texture dimensions");
        byte[] hash = MessageDigest.getInstance("SHA-256").digest(bytes);
        StringBuilder name = new StringBuilder("/");
        for (byte b : hash) name.append(String.format("%02x", b & 255));
        name.append(".png");
        images.put(name.toString(), bytes);
        return name.toString();
    }

    private static String fetchTexture(String name, String asset, String[] variant) throws Exception {
        HttpURLConnection connection = (HttpURLConnection)new URL("https://luxmc-r92.pages.dev/api/appearance/" + name + "?asset=" + asset).openConnection();
        connection.setConnectTimeout(1500);
        connection.setReadTimeout(1500);
        connection.setInstanceFollowRedirects(false);
        try {
            if (connection.getResponseCode() != 200 || connection.getContentLengthLong() > 131072 || !"image/png".equals(connection.getContentType())) return null;
            variant[0] = "slim".equals(connection.getHeaderField("X-Luxmc-Model")) ? "slim" : "default";
            InputStream input = connection.getInputStream();
            ByteArrayOutputStream bytes = new ByteArrayOutputStream();
            byte[] buffer = new byte[4096];
            int count;
            while ((count = input.read(buffer)) != -1) {
                if (bytes.size() + count > 131072) throw new IOException("Texture too large");
                bytes.write(buffer, 0, count);
            }
            return textureBase + addBytes(bytes.toByteArray());
        } finally { connection.disconnect(); }
    }

    private static String officialResponse(String address, boolean image) throws Exception {
        HttpURLConnection connection = (HttpURLConnection)new URL(address).openConnection();
        connection.setConnectTimeout(2000); connection.setReadTimeout(2500); connection.setInstanceFollowRedirects(false);
        connection.setRequestProperty("User-Agent", "Luxmc-Client-Appearance");
        connection.setRequestProperty("Accept", image ? "image/png" : "application/json");
        try {
            if (connection.getResponseCode() != 200 || connection.getContentLengthLong() > 131072) return null;
            ByteArrayOutputStream bytes = new ByteArrayOutputStream();
            try (InputStream input = connection.getInputStream()) {
                byte[] buffer = new byte[4096]; int count;
                while ((count = input.read(buffer)) != -1) { if (bytes.size() + count > 131072) throw new IOException("Response too large"); bytes.write(buffer, 0, count); }
            }
            return image ? textureBase + addBytes(bytes.toByteArray()) : new String(bytes.toByteArray(), StandardCharsets.UTF_8);
        } finally { connection.disconnect(); }
    }

    private static String officialTexture(String payload, String asset) throws Exception {
        Matcher texture = Pattern.compile("\\\"" + asset + "\\\"\\s*:\\s*\\{\\s*\\\"url\\\"\\s*:\\s*\\\"(https?://textures\\.minecraft\\.net/texture/[a-fA-F0-9]+)\\\"").matcher(payload);
        return texture.find() ? officialResponse(texture.group(1).replace("http://", "https://"), true) : null;
    }

    private static RemoteAppearance officialAppearance(String name) throws Exception {
        String profile = officialResponse("https://api.minecraftservices.com/minecraft/profile/lookup/name/" + name, false);
        if (profile == null) return null;
        Matcher uuid = Pattern.compile("\\\"id\\\"\\s*:\\s*\\\"([a-fA-F0-9]{32})\\\"").matcher(profile);
        if (!uuid.find()) return null;
        String session = officialResponse("https://sessionserver.mojang.com/session/minecraft/profile/" + uuid.group(1), false);
        if (session == null) return null;
        Matcher value = Pattern.compile("\\\"value\\\"\\s*:\\s*\\\"([A-Za-z0-9+/=]{1,65536})\\\"").matcher(session);
        if (!value.find()) return null;
        String textures = new String(Base64.getDecoder().decode(value.group(1)), StandardCharsets.UTF_8);
        Matcher profileName = PROFILE_NAME.matcher(textures);
        if (!profileName.find() || !name.equalsIgnoreCase(profileName.group(1))) return null;
        String skin = officialTexture(textures, "SKIN");
        if (skin == null) return null;
        String model = Pattern.compile("\\\"model\\\"\\s*:\\s*\\\"slim\\\"").matcher(textures).find() ? "slim" : "default";
        return new RemoteAppearance(skin, officialTexture(textures, "CAPE"), model, System.currentTimeMillis() + 300000);
    }

    private static RemoteAppearance sharedAppearance(Object input) throws Exception {
        String id = null, name = null;
        if (input.getClass().getName().equals("com.mojang.authlib.GameProfile")) {
            Object profileId = call(input, "getId", "id"), profileName = call(input, "getName", "name");
            if (profileId != null) id = profileId.toString().replace("-", "");
            if (profileName instanceof String) name = (String)profileName;
        } else {
            Object value = call(input, "getValue", "value");
            if (!(value instanceof String) || ((String)value).length() > 65536) return null;
            String payload = new String(Base64.getDecoder().decode((String)value), StandardCharsets.UTF_8);
            Matcher idMatch = PROFILE_ID.matcher(payload), nameMatch = PROFILE_NAME.matcher(payload);
            if (idMatch.find()) id = idMatch.group(1).replace("-", "");
            if (nameMatch.find()) name = nameMatch.group(1);
        }
        if (id == null || name == null || !name.matches("[A-Za-z0-9_]{3,16}")) return null;
        String offlineId = UUID.nameUUIDFromBytes(("OfflinePlayer:" + name).getBytes(StandardCharsets.UTF_8)).toString().replace("-", "");
        if (!offlineId.equalsIgnoreCase(id)) return null;
        RemoteAppearance cached = remote.get(name);
        if (cached != null && cached.expires > System.currentTimeMillis()) return cached;
        if (remote.size() >= 256) return null;
        String[] variant = new String[] {"default"};
        String skin = null, cape = null;
        try {
            skin = fetchTexture(name, "skin", variant);
            if (skin != null) { String[] capeVariant = new String[1]; cape = fetchTexture(name, "cape", capeVariant); }
        } catch (IOException ignored) {}
        if (skin == null) {
            try {
                RemoteAppearance official = officialAppearance(name);
                if (official != null) { remote.put(name, official); return official; }
            } catch (IOException ignored) {}
        }
        RemoteAppearance appearance = new RemoteAppearance(skin, cape, variant[0], System.currentTimeMillis() + (skin == null ? 60000 : 300000));
        remote.put(name, appearance);
        return appearance;
    }

    private static void serve(Socket socket) {
        try {
            socket.setSoTimeout(3000);
            InputStream input = socket.getInputStream();
            ByteArrayOutputStream line = new ByteArrayOutputStream();
            for (int i = 0; i < 1024; i++) { int next = input.read(); if (next < 0 || next == '\n') break; line.write(next); }
            String[] request = new String(line.toByteArray(), StandardCharsets.US_ASCII).trim().split(" ");
            byte[] bytes = request.length == 3 && "GET".equals(request[0]) ? images.get(request[1]) : null;
            OutputStream output = socket.getOutputStream();
            if (bytes == null) output.write("HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".getBytes(StandardCharsets.US_ASCII));
            else {
                output.write(("HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: " + bytes.length + "\r\nConnection: close\r\n\r\n").getBytes(StandardCharsets.US_ASCII));
                output.write(bytes);
            }
            output.flush();
        } catch (IOException ignored) {} finally { try { socket.close(); } catch (IOException ignored) {} }
    }

    private static Object call(Object instance, String... methods) throws Exception {
        for (String method : methods) {
            try { return instance.getClass().getMethod(method).invoke(instance); } catch (NoSuchMethodException ignored) {}
        }
        return null;
    }

    private static boolean owns(Object input) throws Exception {
        if (input == null) return false;
        if (input.getClass().getName().equals("com.mojang.authlib.GameProfile")) {
            Object id = call(input, "getId", "id");
            return id != null && owner.equalsIgnoreCase(id.toString().replace("-", ""));
        }
        Object value = call(input, "getValue", "value");
        if (!(value instanceof String) || ((String)value).length() > 65536) return false;
        String payload = new String(Base64.getDecoder().decode((String)value), StandardCharsets.UTF_8);
        Matcher matcher = PROFILE_ID.matcher(payload);
        return matcher.find() && owner.equalsIgnoreCase(matcher.group(1).replace("-", ""));
    }

    @SuppressWarnings({"rawtypes", "unchecked"})
    public static Object localTextures(Object input, String resultType) {
        try {
            if (owner == null || input == null) return null;
            String selectedSkin = skinUrl, selectedCape = capeUrl, selectedModel = model;
            if (!owns(input) || selectedSkin == null) {
                RemoteAppearance appearance = sharedAppearance(input);
                if (appearance == null || appearance.skin == null) return null;
                selectedSkin = appearance.skin; selectedCape = appearance.cape; selectedModel = appearance.model;
            }
            ClassLoader loader = input.getClass().getClassLoader();
            if (resultType.equals("com/mojang/authlib/properties/Property")) {
                Object profileId = call(input, "getId", "id");
                String selectedId = profileId == null ? owner : profileId.toString().replace("-", "");
                String payload = "{\"profileId\":\"" + selectedId + "\",\"textures\":{\"SKIN\":{\"url\":\"" + selectedSkin + "\",\"metadata\":{\"model\":\"" + selectedModel + "\"}}" + (selectedCape == null ? "" : ",\"CAPE\":{\"url\":\"" + selectedCape + "\"}") + "}}";
                return Class.forName(resultType.replace('/', '.'), true, loader).getConstructor(String.class, String.class).newInstance("textures", Base64.getEncoder().encodeToString(payload.getBytes(StandardCharsets.UTF_8)));
            }
            Class<?> texture = Class.forName("com.mojang.authlib.minecraft.MinecraftProfileTexture", true, loader);
            Constructor<?> create = texture.getConstructor(String.class, Map.class);
            Object skin = create.newInstance(selectedSkin, Collections.singletonMap("model", selectedModel));
            Object cape = selectedCape == null ? null : create.newInstance(selectedCape, Collections.emptyMap());
            if (resultType.equals("java/util/Map")) {
                Class<? extends Enum> type = (Class<? extends Enum>)Class.forName("com.mojang.authlib.minecraft.MinecraftProfileTexture$Type", true, loader);
                Map<Object,Object> map = new HashMap<Object,Object>();
                map.put(Enum.valueOf(type, "SKIN"), skin);
                if (cape != null) map.put(Enum.valueOf(type, "CAPE"), cape);
                return map;
            }
            if (resultType.equals("com/mojang/authlib/minecraft/MinecraftProfileTextures")) {
                Class<? extends Enum> state = (Class<? extends Enum>)Class.forName("com.mojang.authlib.SignatureState", true, loader);
                Class<?> result = Class.forName(resultType.replace('/', '.'), true, loader);
                return result.getConstructor(texture, texture, texture, state).newInstance(skin, cape, null, Enum.valueOf(state, "SIGNED"));
            }
        } catch (Throwable error) { System.err.println("[LUXMC_CLIENT] Local texture adapter failed: " + error.getClass().getSimpleName()); }
        return null;
    }
}
