package io.github.luxmc.client;

import java.awt.*;
import java.awt.event.*;
import java.io.IOException;
import java.lang.instrument.Instrumentation;
import java.lang.reflect.Method;
import java.net.*;
import java.util.*;
import java.util.List;
import javax.swing.*;

public class LuxmcClientAgent {

    private static volatile boolean running = true;
    private static volatile boolean wasRightShiftDown = false;
    private static volatile boolean wasLmbDown = false;
    private static volatile boolean wasRmbDown = false;
    private static long lastToggleTime = 0;

    // Window / Input reflection handles (LWJGL 3)
    private static Method glfwGetCurrentContextMethod = null;
    private static Method glfwGetKeyMethod = null;
    private static Method glfwGetMouseButtonMethod = null;
    private static Method glfwSetInputModeMethod = null;
    private static Method glfwGetWindowPosMethod = null;
    private static Method glfwGetWindowSizeMethod = null;
    private static Method glfwGetWindowAttribMethod = null;
    private static Class<?> glfwClass = null;

    // Window / Input reflection handles (LWJGL 2)
    private static Method isCreatedMethod = null;
    private static Method isKeyDownMethod = null;
    private static Method isButtonDownMethod = null;
    private static Method setGrabbedMethod = null;
    private static Method displayIsActiveMethod = null;
    private static Method displayIsVisibleMethod = null;
    private static Method displayGetXMethod = null;
    private static Method displayGetYMethod = null;
    private static Method displayGetWidthMethod = null;
    private static Method displayGetHeightMethod = null;
    private static Class<?> keyboardClass = null;
    private static Class<?> mouseClass = null;
    private static Class<?> displayClass = null;

    public static volatile Long currentWindowHandle = null;

    // Tracked Minecraft window bounds on desktop
    public static volatile int mcWindowX = 20;
    public static volatile int mcWindowY = 20;
    public static volatile int mcWindowW = 854;
    public static volatile int mcWindowH = 480;
    public static volatile boolean mcWindowVisible = false;

    // In-game GUI & HUD
    private static JDialog inGameMenu = null;
    private static InGameHudWindow hudWindow = null;

    // Module States
    public static volatile boolean modFps = false;
    public static volatile boolean modCps = true;
    public static volatile boolean modKeystrokes = true;
    public static volatile boolean modArmor = false;
    public static volatile boolean modCoords = false;
    public static volatile boolean modPing = false;
    public static volatile boolean modSprint = false;
    public static volatile boolean modDirection = false;
    public static volatile boolean modFullbright = false;

    // Keystroke state
    public static volatile boolean keyW = false;
    public static volatile boolean keyA = false;
    public static volatile boolean keyS = false;
    public static volatile boolean keyD = false;
    public static volatile boolean keyLmb = false;
    public static volatile boolean keyRmb = false;

    public static volatile int leftCps = 0;
    public static volatile int rightCps = 0;
    private static final List<Long> leftClicks = new ArrayList<Long>();
    private static final List<Long> rightClicks = new ArrayList<Long>();

    public static void premain(String agentArgs, Instrumentation inst) {
        System.out.println("[LUXMC_CLIENT] Luxmc Client PvP Suite initialized (v1.9.2)");
        System.out.println("[LUXMC_CLIENT] In-game menu hotkey: Right Shift (Shift Direito)");

        initCapeRedirection();

        Thread hookThread = new Thread(new Runnable() {
            @Override
            public void run() {
                monitorKeyboard();
            }
        }, "Luxmc-Client-Hook");
        hookThread.setDaemon(true);
        hookThread.setPriority(Thread.NORM_PRIORITY);
        hookThread.start();

        SwingUtilities.invokeLater(new Runnable() {
            @Override
            public void run() {
                try {
                    hudWindow = new InGameHudWindow();
                    hudWindow.setVisible(false);
                } catch (Throwable t) {
                    System.out.println("[LUXMC_CLIENT] HUD window init notice: " + t.getMessage());
                }
            }
        });
    }

    public static void agentmain(String agentArgs, Instrumentation inst) {
        premain(agentArgs, inst);
    }

    private static void initCapeRedirection() {
        try {
            final ProxySelector defaultSelector = ProxySelector.getDefault();
            ProxySelector.setDefault(new ProxySelector() {
                @Override
                public List<Proxy> select(URI uri) {
                    if (uri != null && uri.getHost() != null) {
                        String host = uri.getHost().toLowerCase();
                        String path = uri.getPath() != null ? uri.getPath().toLowerCase() : "";
                        if (host.contains("optifine.net") || host.contains("minecraftcapes")
                                || host.contains("cloaksplus") || path.contains("luxmc_cape")
                                || path.contains("/capes/") || path.contains("/cape")) {
                            return Collections.singletonList(new Proxy(Proxy.Type.HTTP, new InetSocketAddress("127.0.0.1", 49152)));
                        }
                    }
                    return defaultSelector != null ? defaultSelector.select(uri) : Collections.singletonList(Proxy.NO_PROXY);
                }

                @Override
                public void connectFailed(URI uri, SocketAddress sa, IOException ioe) {
                    if (defaultSelector != null) {
                        defaultSelector.connectFailed(uri, sa, ioe);
                    }
                }
            });
            System.out.println("[LUXMC_CLIENT] Universal cape redirection active (port 49152)");
        } catch (Throwable t) {
            System.out.println("[LUXMC_CLIENT] Proxy init notice: " + t.getMessage());
        }
    }

    private static Long resolveGlfwWindowHandle() {
        if (currentWindowHandle != null && currentWindowHandle.longValue() != 0L) {
            return currentWindowHandle;
        }

        try {
            if (glfwGetCurrentContextMethod != null) {
                Long h = (Long) glfwGetCurrentContextMethod.invoke(null);
                if (h != null && h.longValue() != 0L) {
                    currentWindowHandle = h;
                    return h;
                }
            }
        } catch (Throwable ignored) {}

        for (Map.Entry<Thread, StackTraceElement[]> entry : Thread.getAllStackTraces().entrySet()) {
            Thread t = entry.getKey();
            String name = t.getName();
            if (name.equals("Render thread") || name.equals("Client thread") || name.equals("main") || name.toLowerCase().contains("minecraft")) {
                ClassLoader cl = t.getContextClassLoader();
                if (cl != null) {
                    try {
                        Class<?> mcClass = cl.loadClass("net.minecraft.client.Minecraft");
                        Method getInstance = mcClass.getMethod("getInstance");
                        Object mc = getInstance.invoke(null);
                        if (mc != null) {
                            for (Method m : mcClass.getMethods()) {
                                if (m.getParameterTypes().length == 0 && (m.getName().equals("getWindow") || m.getReturnType().getName().contains("Window"))) {
                                    Object win = m.invoke(mc);
                                    if (win != null) {
                                        for (Method wm : win.getClass().getMethods()) {
                                            if (wm.getParameterTypes().length == 0 && (wm.getName().equals("getWindow") || wm.getName().equals("getHandle") || wm.getName().equals("handle")) && (wm.getReturnType() == long.class || wm.getReturnType() == Long.class)) {
                                                Long h = (Long) wm.invoke(win);
                                                if (h != null && h.longValue() != 0L) {
                                                    currentWindowHandle = h;
                                                    return h;
                                                }
                                            }
                                        }
                                        for (java.lang.reflect.Field f : win.getClass().getDeclaredFields()) {
                                            if (f.getType() == long.class) {
                                                f.setAccessible(true);
                                                long h = f.getLong(win);
                                                if (h != 0L) {
                                                    currentWindowHandle = Long.valueOf(h);
                                                    return currentWindowHandle;
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            for (Method m : mcClass.getMethods()) {
                                if (m.getParameterTypes().length == 1 && m.getParameterTypes()[0] == Runnable.class) {
                                    m.invoke(mc, new Runnable() {
                                        @Override
                                        public void run() {
                                            try {
                                                if (glfwGetCurrentContextMethod != null) {
                                                    Long h = (Long) glfwGetCurrentContextMethod.invoke(null);
                                                    if (h != null && h.longValue() != 0L) {
                                                        currentWindowHandle = h;
                                                    }
                                                }
                                            } catch (Throwable ignored) {}
                                        }
                                    });
                                    break;
                                }
                            }
                        }
                    } catch (Throwable ignored) {}
                }
            }
        }
        return currentWindowHandle;
    }

    private static void monitorKeyboard() {
        boolean lwjgl2Checked = false;
        boolean lwjgl3Checked = false;

        long lastCpsClean = System.currentTimeMillis();
        long lastHandleSearch = 0;
        int[] posBufX = new int[1];
        int[] posBufY = new int[1];
        int[] sizeBufW = new int[1];
        int[] sizeBufH = new int[1];

        while (running) {
            try {
                Thread.sleep(16);
            } catch (InterruptedException ignored) {
                break;
            }

            boolean isDown = false;
            boolean currentLmb = false;
            boolean currentRmb = false;

            if (!lwjgl2Checked) {
                try {
                    keyboardClass = Class.forName("org.lwjgl.input.Keyboard");
                    isCreatedMethod = keyboardClass.getMethod("isCreated");
                    isKeyDownMethod = keyboardClass.getMethod("isKeyDown", int.class);

                    mouseClass = Class.forName("org.lwjgl.input.Mouse");
                    isButtonDownMethod = mouseClass.getMethod("isButtonDown", int.class);
                    setGrabbedMethod = mouseClass.getMethod("setGrabbed", boolean.class);

                    displayClass = Class.forName("org.lwjgl.opengl.Display");
                    displayIsActiveMethod = displayClass.getMethod("isActive");
                    displayIsVisibleMethod = displayClass.getMethod("isVisible");
                    displayGetXMethod = displayClass.getMethod("getX");
                    displayGetYMethod = displayClass.getMethod("getY");
                    displayGetWidthMethod = displayClass.getMethod("getWidth");
                    displayGetHeightMethod = displayClass.getMethod("getHeight");

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
                        Boolean down = (Boolean) isKeyDownMethod.invoke(null, 54);
                        if (down != null && down.booleanValue()) {
                            isDown = true;
                        }

                        Boolean wDown = (Boolean) isKeyDownMethod.invoke(null, 17);
                        Boolean aDown = (Boolean) isKeyDownMethod.invoke(null, 30);
                        Boolean sDown = (Boolean) isKeyDownMethod.invoke(null, 31);
                        Boolean dDown = (Boolean) isKeyDownMethod.invoke(null, 32);
                        keyW = wDown != null && wDown.booleanValue();
                        keyA = aDown != null && aDown.booleanValue();
                        keyS = sDown != null && sDown.booleanValue();
                        keyD = dDown != null && dDown.booleanValue();

                        if (isButtonDownMethod != null) {
                            Boolean lmb = (Boolean) isButtonDownMethod.invoke(null, 0);
                            Boolean rmb = (Boolean) isButtonDownMethod.invoke(null, 1);
                            currentLmb = lmb != null && lmb.booleanValue();
                            currentRmb = rmb != null && rmb.booleanValue();
                        }

                        if (displayClass != null && displayIsVisibleMethod != null) {
                            Boolean v = (Boolean) displayIsVisibleMethod.invoke(null);
                            Boolean a = displayIsActiveMethod != null ? (Boolean) displayIsActiveMethod.invoke(null) : Boolean.TRUE;
                            mcWindowVisible = (v != null && v.booleanValue()) && (a != null && a.booleanValue());
                            if (displayGetXMethod != null && displayGetYMethod != null) {
                                mcWindowX = (Integer) displayGetXMethod.invoke(null);
                                mcWindowY = (Integer) displayGetYMethod.invoke(null);
                                mcWindowW = (Integer) displayGetWidthMethod.invoke(null);
                                mcWindowH = (Integer) displayGetHeightMethod.invoke(null);
                            }
                        }
                    }
                } catch (Throwable ignored) {}
            }

            if (!isDown) {
                if (!lwjgl3Checked) {
                    try {
                        glfwClass = Class.forName("org.lwjgl.glfw.GLFW");
                        glfwGetCurrentContextMethod = glfwClass.getMethod("glfwGetCurrentContext");
                        glfwGetKeyMethod = glfwClass.getMethod("glfwGetKey", long.class, int.class);
                        glfwGetMouseButtonMethod = glfwClass.getMethod("glfwGetMouseButton", long.class, int.class);
                        glfwSetInputModeMethod = glfwClass.getMethod("glfwSetInputMode", long.class, int.class, int.class);
                        glfwGetWindowPosMethod = glfwClass.getMethod("glfwGetWindowPos", long.class, int[].class, int[].class);
                        glfwGetWindowSizeMethod = glfwClass.getMethod("glfwGetWindowSize", long.class, int[].class, int[].class);
                        glfwGetWindowAttribMethod = glfwClass.getMethod("glfwGetWindowAttrib", long.class, int.class);
                        lwjgl3Checked = true;
                        System.out.println("[LUXMC_CLIENT] Detected LWJGL 3 runtime (Modern mode)");
                    } catch (Throwable t) {
                        lwjgl3Checked = true;
                    }
                }

                long now = System.currentTimeMillis();
                if (currentWindowHandle == null && now - lastHandleSearch > 1000) {
                    lastHandleSearch = now;
                    resolveGlfwWindowHandle();
                }

                if (glfwClass != null && currentWindowHandle != null && currentWindowHandle.longValue() != 0L) {
                    try {
                        long handle = currentWindowHandle.longValue();

                        if (glfwGetKeyMethod != null) {
                            Integer state = (Integer) glfwGetKeyMethod.invoke(null, handle, 344);
                            if (state != null && state.intValue() == 1) {
                                isDown = true;
                            }

                            Integer stateW = (Integer) glfwGetKeyMethod.invoke(null, handle, 87);
                            Integer stateA = (Integer) glfwGetKeyMethod.invoke(null, handle, 65);
                            Integer stateS = (Integer) glfwGetKeyMethod.invoke(null, handle, 83);
                            Integer stateD = (Integer) glfwGetKeyMethod.invoke(null, handle, 68);
                            keyW = stateW != null && stateW.intValue() == 1;
                            keyA = stateA != null && stateA.intValue() == 1;
                            keyS = stateS != null && stateS.intValue() == 1;
                            keyD = stateD != null && stateD.intValue() == 1;
                        }

                        if (glfwGetMouseButtonMethod != null) {
                            Integer btn0 = (Integer) glfwGetMouseButtonMethod.invoke(null, handle, 0);
                            Integer btn1 = (Integer) glfwGetMouseButtonMethod.invoke(null, handle, 1);
                            currentLmb = btn0 != null && btn0.intValue() == 1;
                            currentRmb = btn1 != null && btn1.intValue() == 1;
                        }

                        if (glfwGetWindowPosMethod != null) {
                            glfwGetWindowPosMethod.invoke(null, handle, posBufX, posBufY);
                            mcWindowX = posBufX[0];
                            mcWindowY = posBufY[0];
                        }
                        if (glfwGetWindowSizeMethod != null) {
                            glfwGetWindowSizeMethod.invoke(null, handle, sizeBufW, sizeBufH);
                            mcWindowW = sizeBufW[0];
                            mcWindowH = sizeBufH[0];
                        }
                        if (glfwGetWindowAttribMethod != null) {
                            Integer iconified = (Integer) glfwGetWindowAttribMethod.invoke(null, handle, 0x00020002);
                            Integer visible = (Integer) glfwGetWindowAttribMethod.invoke(null, handle, 0x00020004);
                            mcWindowVisible = (visible != null && visible.intValue() == 1)
                                    && (iconified == null || iconified.intValue() == 0);
                        }
                    } catch (Throwable ignored) {}
                }
            }

            keyLmb = currentLmb;
            keyRmb = currentRmb;

            if (currentLmb && !wasLmbDown) {
                recordClick(false);
            }
            wasLmbDown = currentLmb;

            if (currentRmb && !wasRmbDown) {
                recordClick(true);
            }
            wasRmbDown = currentRmb;

            long now = System.currentTimeMillis();
            if (now - lastCpsClean > 150) {
                lastCpsClean = now;
                cleanCps(now);
            }

            if (isDown && !wasRightShiftDown && (now - lastToggleTime > 350)) {
                lastToggleTime = now;
                wasRightShiftDown = true;
                toggleInGameMenu();
            } else if (!isDown) {
                wasRightShiftDown = false;
            }
        }
    }

    private static synchronized void cleanCps(long now) {
        long cutoff = now - 1000;
        Iterator<Long> itL = leftClicks.iterator();
        while (itL.hasNext()) {
            if (itL.next().longValue() < cutoff) itL.remove();
        }
        leftCps = leftClicks.size();

        Iterator<Long> itR = rightClicks.iterator();
        while (itR.hasNext()) {
            if (itR.next().longValue() < cutoff) itR.remove();
        }
        rightCps = rightClicks.size();
    }

    public static synchronized void recordClick(boolean right) {
        long now = System.currentTimeMillis();
        if (right) {
            rightClicks.add(Long.valueOf(now));
            keyRmb = true;
        } else {
            leftClicks.add(Long.valueOf(now));
            keyLmb = true;
        }
    }

    public static void releaseMouse() {
        try {
            if (glfwClass != null && glfwSetInputModeMethod != null && currentWindowHandle != null && currentWindowHandle.longValue() != 0L) {
                glfwSetInputModeMethod.invoke(null, currentWindowHandle.longValue(), 0x00033001, 0x00034001);
            } else if (mouseClass != null && setGrabbedMethod != null) {
                setGrabbedMethod.invoke(null, Boolean.FALSE);
            }
        } catch (Throwable ignored) {}
    }

    public static void restoreMouse() {
        try {
            if (glfwClass != null && glfwSetInputModeMethod != null && currentWindowHandle != null && currentWindowHandle.longValue() != 0L) {
                glfwSetInputModeMethod.invoke(null, currentWindowHandle.longValue(), 0x00033001, 0x00034003);
            } else if (mouseClass != null && setGrabbedMethod != null) {
                setGrabbedMethod.invoke(null, Boolean.TRUE);
            }
        } catch (Throwable ignored) {}
    }

    private static void toggleInGameMenu() {
        SwingUtilities.invokeLater(new Runnable() {
            @Override
            public void run() {
                if (inGameMenu != null && inGameMenu.isVisible()) {
                    inGameMenu.setVisible(false);
                    restoreMouse();
                } else {
                    releaseMouse();
                    if (inGameMenu == null) {
                        inGameMenu = createInGameMenuDialog();
                    }
                    int menuW = inGameMenu.getWidth();
                    int menuH = inGameMenu.getHeight();
                    int posX = Math.max(0, mcWindowX + (mcWindowW - menuW) / 2);
                    int posY = Math.max(0, mcWindowY + (mcWindowH - menuH) / 2);
                    inGameMenu.setLocation(posX, posY);
                    inGameMenu.setVisible(true);
                    inGameMenu.toFront();
                }
            }
        });
    }

    private static JDialog createInGameMenuDialog() {
        final JDialog dialog = new JDialog((Frame) null, false);
        dialog.setType(Window.Type.POPUP);
        dialog.setUndecorated(true);
        dialog.setSize(540, 440);
        dialog.setAlwaysOnTop(true);

        JPanel mainPanel = new JPanel(new BorderLayout());
        mainPanel.setBackground(new Color(11, 15, 25));
        mainPanel.setBorder(BorderFactory.createCompoundBorder(
                BorderFactory.createLineBorder(new Color(37, 99, 235), 2),
                BorderFactory.createEmptyBorder(20, 24, 20, 24)
        ));

        // Header
        JPanel headerPanel = new JPanel(new BorderLayout());
        headerPanel.setOpaque(false);

        JLabel titleLabel = new JLabel("LUXMC CLIENT PVP SUITE");
        titleLabel.setFont(new Font("SansSerif", Font.BOLD, 18));
        titleLabel.setForeground(new Color(59, 130, 246));

        JLabel subLabel = new JLabel("Menu In-Game · Clique para alternar módulos · Shift Direito ou ESC para fechar");
        subLabel.setFont(new Font("SansSerif", Font.PLAIN, 12));
        subLabel.setForeground(new Color(148, 163, 184));

        headerPanel.add(titleLabel, BorderLayout.NORTH);
        headerPanel.add(subLabel, BorderLayout.SOUTH);
        mainPanel.add(headerPanel, BorderLayout.NORTH);

        // Modules Grid
        JPanel gridPanel = new JPanel(new GridLayout(3, 3, 12, 12));
        gridPanel.setOpaque(false);
        gridPanel.setBorder(BorderFactory.createEmptyBorder(18, 0, 18, 0));

        gridPanel.add(createModuleToggle("CPS Display", modCps, new ActionListener() {
            @Override
            public void actionPerformed(ActionEvent e) {
                modCps = !modCps;
                updateButtonState((JButton) e.getSource(), modCps);
            }
        }));

        gridPanel.add(createModuleToggle("Keystrokes", modKeystrokes, new ActionListener() {
            @Override
            public void actionPerformed(ActionEvent e) {
                modKeystrokes = !modKeystrokes;
                updateButtonState((JButton) e.getSource(), modKeystrokes);
            }
        }));

        gridPanel.add(createModuleToggle("FPS Display", modFps, new ActionListener() {
            @Override
            public void actionPerformed(ActionEvent e) {
                modFps = !modFps;
                updateButtonState((JButton) e.getSource(), modFps);
            }
        }));

        gridPanel.add(createModuleToggle("Armor HUD", modArmor, new ActionListener() {
            @Override
            public void actionPerformed(ActionEvent e) {
                modArmor = !modArmor;
                updateButtonState((JButton) e.getSource(), modArmor);
            }
        }));

        gridPanel.add(createModuleToggle("Coordenadas", modCoords, new ActionListener() {
            @Override
            public void actionPerformed(ActionEvent e) {
                modCoords = !modCoords;
                updateButtonState((JButton) e.getSource(), modCoords);
            }
        }));

        gridPanel.add(createModuleToggle("Ping Display", modPing, new ActionListener() {
            @Override
            public void actionPerformed(ActionEvent e) {
                modPing = !modPing;
                updateButtonState((JButton) e.getSource(), modPing);
            }
        }));

        gridPanel.add(createModuleToggle("Toggle Sprint", modSprint, new ActionListener() {
            @Override
            public void actionPerformed(ActionEvent e) {
                modSprint = !modSprint;
                updateButtonState((JButton) e.getSource(), modSprint);
            }
        }));

        gridPanel.add(createModuleToggle("Full Bright", modFullbright, new ActionListener() {
            @Override
            public void actionPerformed(ActionEvent e) {
                modFullbright = !modFullbright;
                updateButtonState((JButton) e.getSource(), modFullbright);
            }
        }));

        gridPanel.add(createModuleToggle("Direção HUD", modDirection, new ActionListener() {
            @Override
            public void actionPerformed(ActionEvent e) {
                modDirection = !modDirection;
                updateButtonState((JButton) e.getSource(), modDirection);
            }
        }));

        mainPanel.add(gridPanel, BorderLayout.CENTER);

        // Footer
        JButton closeButton = new JButton("Salvar e Fechar (ESC)");
        closeButton.setFont(new Font("SansSerif", Font.BOLD, 13));
        closeButton.setBackground(new Color(37, 99, 235));
        closeButton.setForeground(Color.WHITE);
        closeButton.setFocusPainted(false);
        closeButton.setBorder(BorderFactory.createEmptyBorder(12, 0, 12, 0));
        closeButton.setCursor(Cursor.getPredefinedCursor(Cursor.HAND_CURSOR));
        closeButton.addActionListener(new ActionListener() {
            @Override
            public void actionPerformed(ActionEvent e) {
                dialog.setVisible(false);
                restoreMouse();
            }
        });

        mainPanel.add(closeButton, BorderLayout.SOUTH);
        dialog.setContentPane(mainPanel);

        dialog.addWindowListener(new WindowAdapter() {
            @Override
            public void windowClosed(WindowEvent e) {
                restoreMouse();
            }
        });

        ActionListener closeAction = new ActionListener() {
            @Override
            public void actionPerformed(ActionEvent e) {
                dialog.setVisible(false);
                restoreMouse();
            }
        };

        dialog.getRootPane().registerKeyboardAction(closeAction, KeyStroke.getKeyStroke(KeyEvent.VK_ESCAPE, 0), JComponent.WHEN_IN_FOCUSED_WINDOW);
        dialog.getRootPane().registerKeyboardAction(closeAction, KeyStroke.getKeyStroke(KeyEvent.VK_SHIFT, 0), JComponent.WHEN_IN_FOCUSED_WINDOW);

        return dialog;
    }

    private static JButton createModuleToggle(String label, boolean active, ActionListener listener) {
        final JButton button = new JButton();
        button.setLayout(new BorderLayout());
        button.setFocusPainted(false);
        button.setCursor(Cursor.getPredefinedCursor(Cursor.HAND_CURSOR));
        button.addActionListener(listener);

        updateButtonState(button, active);
        button.setText(label + (active ? " [ON]" : " [OFF]"));
        return button;
    }

    private static void updateButtonState(JButton button, boolean active) {
        if (active) {
            button.setBackground(new Color(37, 99, 235, 240));
            button.setForeground(Color.WHITE);
            button.setFont(new Font("SansSerif", Font.BOLD, 12));
            button.setBorder(BorderFactory.createLineBorder(new Color(96, 165, 250), 1));
        } else {
            button.setBackground(new Color(19, 25, 39));
            button.setForeground(new Color(148, 163, 184));
            button.setFont(new Font("SansSerif", Font.PLAIN, 12));
            button.setBorder(BorderFactory.createLineBorder(new Color(37, 47, 66), 1));
        }
        String text = button.getText();
        if (text != null && !text.isEmpty()) {
            String base = text.replaceAll(" \\[ON\\]| \\[OFF\\]", "");
            button.setText(base + (active ? " [ON]" : " [OFF]"));
        }
    }

    public static class InGameHudWindow extends JWindow {
        public InGameHudWindow() {
            setType(Window.Type.POPUP);
            boolean canTranslucent = false;
            try {
                GraphicsEnvironment ge = GraphicsEnvironment.getLocalGraphicsEnvironment();
                GraphicsDevice gd = ge.getDefaultScreenDevice();
                canTranslucent = gd.isWindowTranslucencySupported(GraphicsDevice.WindowTranslucency.TRANSLUCENT);
                if (canTranslucent) {
                    setBackground(new Color(0, 0, 0, 0));
                }
            } catch (Throwable ignored) {}
            setAlwaysOnTop(true);
            setFocusableWindowState(false);
            setFocusable(false);
            setAutoRequestFocus(false);
            setSize(200, 220);
            setLocation(mcWindowX + 16, mcWindowY + 36);
            setVisible(false);

            final boolean supported = canTranslucent;
            javax.swing.Timer timer = new javax.swing.Timer(30, new ActionListener() {
                @Override
                public void actionPerformed(ActionEvent e) {
                    if (!supported) {
                        if (isVisible()) setVisible(false);
                        return;
                    }
                    boolean hasWindow = (currentWindowHandle != null && currentWindowHandle.longValue() != 0L)
                            || (keyboardClass != null);
                    boolean active = mcWindowVisible && mcWindowW > 150 && mcWindowH > 150 && hasWindow;

                    if (!active || (!modCps && !modKeystrokes)) {
                        if (isVisible()) setVisible(false);
                    } else {
                        if (!isVisible()) setVisible(true);
                        int targetX = Math.max(0, mcWindowX + 16);
                        int targetY = Math.max(0, mcWindowY + 36);
                        if (getX() != targetX || getY() != targetY) {
                            setLocation(targetX, targetY);
                        }
                        repaint();
                    }
                }
            });
            timer.start();
        }

        @Override
        public void paint(Graphics g) {
            super.paint(g);
            Graphics2D g2 = (Graphics2D) g.create();
            g2.setComposite(AlphaComposite.Src);
            g2.setColor(new Color(0, 0, 0, 0));
            g2.fillRect(0, 0, getWidth(), getHeight());
            g2.setComposite(AlphaComposite.SrcOver);

            g2.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
            g2.setRenderingHint(RenderingHints.KEY_TEXT_ANTIALIASING, RenderingHints.VALUE_TEXT_ANTIALIAS_ON);
            g2.setRenderingHint(RenderingHints.KEY_RENDERING, RenderingHints.VALUE_RENDER_QUALITY);
            g2.setRenderingHint(RenderingHints.KEY_STROKE_CONTROL, RenderingHints.VALUE_STROKE_PURE);

            int y = 4;

            if (modCps) {
                drawCpsPill(g2, 4, y);
                y += 32;
            }

            if (modKeystrokes) {
                drawKeystrokes(g2, 4, y);
            }

            g2.dispose();
        }

        private void drawCpsPill(Graphics2D g2, int x, int y) {
            g2.setColor(new Color(11, 15, 25, 215));
            g2.fillRoundRect(x, y, 148, 26, 8, 8);
            g2.setColor(new Color(59, 130, 246, 90));
            g2.drawRoundRect(x, y, 148, 26, 8, 8);

            g2.setColor(new Color(56, 189, 248));
            g2.fillOval(x + 8, y + 9, 7, 7);

            g2.setFont(new Font("SansSerif", Font.BOLD, 11));
            g2.setColor(Color.WHITE);
            g2.drawString("CPS:", x + 20, y + 17);

            g2.setFont(new Font("Monospaced", Font.BOLD, 11));
            g2.setColor(new Color(56, 189, 248));
            g2.drawString(leftCps + " LMB", x + 50, y + 17);
            g2.setColor(new Color(148, 163, 184));
            g2.drawString("|", x + 98, y + 17);
            g2.setColor(new Color(56, 189, 248));
            g2.drawString(rightCps + " RMB", x + 107, y + 17);
        }

        private void drawKeystrokes(Graphics2D g2, int x, int y) {
            drawKeyBox(g2, x + 34, y, "W", keyW);
            drawKeyBox(g2, x, y + 34, "A", keyA);
            drawKeyBox(g2, x + 34, y + 34, "S", keyS);
            drawKeyBox(g2, x + 68, y + 34, "D", keyD);

            int btnW = 49;
            int btnH = 30;
            int btnY = y + 68;

            drawMouseBox(g2, x, btnY, btnW, btnH, "LMB", leftCps, keyLmb);
            drawMouseBox(g2, x + 53, btnY, btnW, btnH, "RMB", rightCps, keyRmb);
        }

        private void drawKeyBox(Graphics2D g2, int x, int y, String key, boolean pressed) {
            if (pressed) {
                g2.setColor(new Color(37, 99, 235, 235));
                g2.fillRoundRect(x, y, 30, 30, 8, 8);
                g2.setColor(new Color(147, 197, 253));
                g2.drawRoundRect(x, y, 30, 30, 8, 8);
                g2.setColor(Color.WHITE);
            } else {
                g2.setColor(new Color(11, 15, 25, 210));
                g2.fillRoundRect(x, y, 30, 30, 8, 8);
                g2.setColor(new Color(255, 255, 255, 30));
                g2.drawRoundRect(x, y, 30, 30, 8, 8);
                g2.setColor(new Color(203, 213, 225, 210));
            }
            g2.setFont(new Font("SansSerif", Font.BOLD, 13));
            FontMetrics fm = g2.getFontMetrics();
            int tx = x + (30 - fm.stringWidth(key)) / 2;
            int ty = y + (30 - fm.getHeight()) / 2 + fm.getAscent();
            g2.drawString(key, tx, ty);
        }

        private void drawMouseBox(Graphics2D g2, int x, int y, int w, int h, String btn, int cps, boolean pressed) {
            if (pressed) {
                g2.setColor(new Color(37, 99, 235, 235));
                g2.fillRoundRect(x, y, w, h, 8, 8);
                g2.setColor(new Color(147, 197, 253));
                g2.drawRoundRect(x, y, w, h, 8, 8);
                g2.setColor(Color.WHITE);
            } else {
                g2.setColor(new Color(11, 15, 25, 210));
                g2.fillRoundRect(x, y, w, h, 8, 8);
                g2.setColor(new Color(255, 255, 255, 30));
                g2.drawRoundRect(x, y, w, h, 8, 8);
                g2.setColor(new Color(203, 213, 225, 210));
            }
            g2.setFont(new Font("SansSerif", Font.BOLD, 10));
            FontMetrics fm = g2.getFontMetrics();
            int tx = x + (w - fm.stringWidth(btn)) / 2;
            g2.drawString(btn, tx, y + 13);

            g2.setFont(new Font("Monospaced", Font.BOLD, 9));
            FontMetrics fmCps = g2.getFontMetrics();
            String cpsStr = cps + " CPS";
            int cx = x + (w - fmCps.stringWidth(cpsStr)) / 2;
            if (pressed) {
                g2.setColor(new Color(224, 242, 254));
            } else {
                g2.setColor(new Color(148, 163, 184));
            }
            g2.drawString(cpsStr, cx, y + 25);
        }
    }
}
