package io.github.luxmc.client;

import java.lang.instrument.Instrumentation;
import java.lang.reflect.Method;
import java.net.Socket;
import java.io.OutputStream;

public class LuxmcClientAgent {

    private static volatile boolean running = true;
    private static volatile boolean wasRightShiftDown = false;
    private static long lastToggleTime = 0;

    public static void premain(String agentArgs, Instrumentation inst) {
        System.out.println("[LUXMC_CLIENT] Luxmc Client PvP Suite initialized (v1.7.6)");
        System.out.println("[LUXMC_CLIENT] In-game menu hotkey: Right Shift (Shift Direito)");

        Thread hookThread = new Thread(new Runnable() {
            @Override
            public void run() {
                monitorKeyboard();
            }
        }, "Luxmc-Client-Hook");
        hookThread.setDaemon(true);
        hookThread.setPriority(Thread.NORM_PRIORITY);
        hookThread.start();
    }

    public static void agentmain(String agentArgs, Instrumentation inst) {
        premain(agentArgs, inst);
    }

    private static void monitorKeyboard() {
        Method isCreatedMethod = null;
        Method isKeyDownMethod = null;
        Class<?> keyboardClass = null;

        Method glfwGetCurrentContextMethod = null;
        Method glfwGetKeyMethod = null;
        Class<?> glfwClass = null;

        boolean lwjgl2Checked = false;
        boolean lwjgl3Checked = false;

        while (running) {
            try {
                Thread.sleep(16); // ~60 Hz polling
            } catch (InterruptedException ignored) {
                break;
            }

            boolean isDown = false;

            // 1. Try LWJGL 2 (Minecraft 1.8.9 - 1.12.2)
            if (!lwjgl2Checked) {
                try {
                    keyboardClass = Class.forName("org.lwjgl.input.Keyboard");
                    isCreatedMethod = keyboardClass.getMethod("isCreated");
                    isKeyDownMethod = keyboardClass.getMethod("isKeyDown", int.class);
                    lwjgl2Checked = true;
                    System.out.println("[LUXMC_CLIENT] Detected LWJGL 2 runtime (1.8.9 PvP mode)");
                } catch (Throwable t) {
                    lwjgl2Checked = true;
                }
            }

            if (keyboardClass != null && isCreatedMethod != null && isKeyDownMethod != null) {
                try {
                    Boolean created = (Boolean) isCreatedMethod.invoke(null);
                    if (created != null && created.booleanValue()) {
                        // 54 = Keyboard.KEY_RSHIFT
                        Boolean down = (Boolean) isKeyDownMethod.invoke(null, 54);
                        if (down != null && down.booleanValue()) {
                            isDown = true;
                        }
                    }
                } catch (Throwable ignored) {}
            }

            // 2. Try LWJGL 3 (Minecraft 1.13+)
            if (!isDown) {
                if (!lwjgl3Checked) {
                    try {
                        glfwClass = Class.forName("org.lwjgl.glfw.GLFW");
                        glfwGetCurrentContextMethod = glfwClass.getMethod("glfwGetCurrentContext");
                        glfwGetKeyMethod = glfwClass.getMethod("glfwGetKey", long.class, int.class);
                        lwjgl3Checked = true;
                        System.out.println("[LUXMC_CLIENT] Detected LWJGL 3 runtime (Modern mode)");
                    } catch (Throwable t) {
                        lwjgl3Checked = true;
                    }
                }

                if (glfwClass != null && glfwGetCurrentContextMethod != null && glfwGetKeyMethod != null) {
                    try {
                        Long windowHandle = (Long) glfwGetCurrentContextMethod.invoke(null);
                        if (windowHandle != null && windowHandle.longValue() != 0L) {
                            // 344 = GLFW_KEY_RIGHT_SHIFT
                            Integer state = (Integer) glfwGetKeyMethod.invoke(null, windowHandle.longValue(), 344);
                            if (state != null && state.intValue() == 1) {
                                isDown = true;
                            }
                        }
                    } catch (Throwable ignored) {}
                }
            }

            long now = System.currentTimeMillis();
            if (isDown && !wasRightShiftDown && (now - lastToggleTime > 250)) {
                lastToggleTime = now;
                wasRightShiftDown = true;
                triggerToggle();
            } else if (!isDown) {
                wasRightShiftDown = false;
            }
        }
    }

    private static void triggerToggle() {
        System.out.println("[LUXMC_CLIENT] TOGGLE_OVERLAY");
        System.out.flush();

        // Also ping TCP port 49152 on localhost (non-blocking)
        try {
            Socket socket = new Socket("127.0.0.1", 49152);
            socket.setSoTimeout(100);
            OutputStream out = socket.getOutputStream();
            out.write("TOGGLE\n".getBytes("UTF-8"));
            out.flush();
            out.close();
            socket.close();
        } catch (Throwable ignored) {
            // Stdout reader in Rust is the primary listener
        }
    }
}
