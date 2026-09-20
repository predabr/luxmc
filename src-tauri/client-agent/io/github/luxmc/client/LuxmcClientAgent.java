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
    private static long lastToggleTime = 0;

    // Window / Input reflection handles
    private static Method glfwGetCurrentContextMethod = null;
    private static Method glfwGetKeyMethod = null;
    private static Method glfwSetInputModeMethod = null;
    private static Method glfwGetWindowPosMethod = null;
    private static Method glfwGetWindowSizeMethod = null;
    private static Class<?> glfwClass = null;

    private static Method isKeyDownMethod = null;
    private static Method setGrabbedMethod = null;
    private static Class<?> keyboardClass = null;
    private static Class<?> mouseClass = null;

    private static Long currentWindowHandle = null;

    // In-game GUI & HUD
    private static JDialog inGameMenu = null;
    private static InGameHudWindow hudWindow = null;

    // Module States
    public static volatile boolean modFps = true;
    public static volatile boolean modCps = true;
    public static volatile boolean modKeystrokes = true;
    public static volatile boolean modArmor = true;
    public static volatile boolean modCoords = true;
    public static volatile boolean modPing = true;
    public static volatile boolean modSprint = true;
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

        // 1. Cape & Texture network redirection proxy
        initCapeRedirection();

        // 2. Start keyboard and input monitor thread
        Thread hookThread = new Thread(new Runnable() {
            @Override
            public void run() {
                monitorKeyboard();
            }
        }, "Luxmc-Client-Hook");
        hookThread.setDaemon(true);
        hookThread.setPriority(Thread.NORM_PRIORITY);
        hookThread.start();

        // 3. Initialize HUD overlay on Swing thread
        SwingUtilities.invokeLater(new Runnable() {
            @Override
            public void run() {
                try {
                    hudWindow = new InGameHudWindow();
                    hudWindow.setVisible(true);
                } catch (Throwable t) {
                    System.out.println("[LUXMC_CLIENT] HUD window init note: " + t.getMessage());
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

    private static void monitorKeyboard() {
        Method isCreatedMethod = null;
        Method glfwGetCurrentContext = null;

        boolean lwjgl2Checked = false;
        boolean lwjgl3Checked = false;

        long lastCpsClean = System.currentTimeMillis();

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

                    mouseClass = Class.forName("org.lwjgl.input.Mouse");
                    setGrabbedMethod = mouseClass.getMethod("setGrabbed", boolean.class);

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

                        // WASD for keystrokes
                        Boolean wDown = (Boolean) isKeyDownMethod.invoke(null, 17); // KEY_W
                        Boolean aDown = (Boolean) isKeyDownMethod.invoke(null, 30); // KEY_A
                        Boolean sDown = (Boolean) isKeyDownMethod.invoke(null, 31); // KEY_S
                        Boolean dDown = (Boolean) isKeyDownMethod.invoke(null, 32); // KEY_D
                        keyW = wDown != null && wDown.booleanValue();
                        keyA = aDown != null && aDown.booleanValue();
                        keyS = sDown != null && sDown.booleanValue();
                        keyD = dDown != null && dDown.booleanValue();
                    }
                } catch (Throwable ignored) {}
            }

            // 2. Try LWJGL 3 (Minecraft 1.13+)
            if (!isDown) {
                if (!lwjgl3Checked) {
                    try {
                        glfwClass = Class.forName("org.lwjgl.glfw.GLFW");
                        glfwGetCurrentContext = glfwClass.getMethod("glfwGetCurrentContext");
                        glfwGetKeyMethod = glfwClass.getMethod("glfwGetKey", long.class, int.class);
                        glfwSetInputModeMethod = glfwClass.getMethod("glfwSetInputMode", long.class, int.class, int.class);
                        glfwGetWindowPosMethod = glfwClass.getMethod("glfwGetWindowPos", long.class, int[].class, int[].class);
                        glfwGetWindowSizeMethod = glfwClass.getMethod("glfwGetWindowSize", long.class, int[].class, int[].class);
                        lwjgl3Checked = true;
                        System.out.println("[LUXMC_CLIENT] Detected LWJGL 3 runtime (Modern mode)");
                    } catch (Throwable t) {
                        lwjgl3Checked = true;
                    }
                }

                if (glfwClass != null && glfwGetCurrentContext != null && glfwGetKeyMethod != null) {
                    try {
                        Long windowHandle = (Long) glfwGetCurrentContext.invoke(null);
                        if (windowHandle != null && windowHandle.longValue() != 0L) {
                            currentWindowHandle = windowHandle;
                            // 344 = GLFW_KEY_RIGHT_SHIFT
                            Integer state = (Integer) glfwGetKeyMethod.invoke(null, windowHandle.longValue(), 344);
                            if (state != null && state.intValue() == 1) {
                                isDown = true;
                            }

                            // WASD keys
                            Integer stateW = (Integer) glfwGetKeyMethod.invoke(null, windowHandle.longValue(), 87);
                            Integer stateA = (Integer) glfwGetKeyMethod.invoke(null, windowHandle.longValue(), 65);
                            Integer stateS = (Integer) glfwGetKeyMethod.invoke(null, windowHandle.longValue(), 83);
                            Integer stateD = (Integer) glfwGetKeyMethod.invoke(null, windowHandle.longValue(), 68);
                            keyW = stateW != null && stateW.intValue() == 1;
                            keyA = stateA != null && stateA.intValue() == 1;
                            keyS = stateS != null && stateS.intValue() == 1;
                            keyD = stateD != null && stateD.intValue() == 1;
                        }
                    } catch (Throwable ignored) {}
                }
            }

            // Clean CPS lists once per second
            long now = System.currentTimeMillis();
            if (now - lastCpsClean > 200) {
                lastCpsClean = now;
                cleanCps(now);
            }

            if (isDown && !wasRightShiftDown && (now - lastToggleTime > 400)) {
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
            if (glfwClass != null && glfwSetInputModeMethod != null && currentWindowHandle != null) {
                // GLFW_CURSOR = 0x00033001, GLFW_CURSOR_NORMAL = 0x00034001
                glfwSetInputModeMethod.invoke(null, currentWindowHandle.longValue(), 0x00033001, 0x00034001);
            } else if (mouseClass != null && setGrabbedMethod != null) {
                setGrabbedMethod.invoke(null, Boolean.FALSE);
            }
        } catch (Throwable ignored) {}
    }

    public static void restoreMouse() {
        try {
            if (glfwClass != null && glfwSetInputModeMethod != null && currentWindowHandle != null) {
                // GLFW_CURSOR_DISABLED = 0x00034003
                glfwSetInputModeMethod.invoke(null, currentWindowHandle.longValue(), 0x00033001, 0x00034003);
            } else if (mouseClass != null && setGrabbedMethod != null) {
                setGrabbedMethod.invoke(null, Boolean.TRUE);
            }
        } catch (Throwable ignored) {}
    }

    private static void toggleInGameMenu() {
        System.out.println("[LUXMC_CLIENT] In-game menu toggle triggered inside Minecraft");
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
                    inGameMenu.setLocationRelativeTo(null);
                    inGameMenu.setVisible(true);
                    inGameMenu.toFront();
                }
            }
        });
    }

    private static JDialog createInGameMenuDialog() {
        final JDialog dialog = new JDialog((Frame) null, false);
        dialog.setUndecorated(true);
        dialog.setSize(480, 440);
        dialog.setAlwaysOnTop(true);

        JPanel mainPanel = new JPanel(new BorderLayout());
        mainPanel.setBackground(new Color(11, 15, 25));
        mainPanel.setBorder(BorderFactory.createCompoundBorder(
                BorderFactory.createLineBorder(new Color(59, 130, 246), 2),
                BorderFactory.createEmptyBorder(16, 20, 16, 20)
        ));

        // Header
        JPanel headerPanel = new JPanel(new BorderLayout());
        headerPanel.setOpaque(false);

        JLabel titleLabel = new JLabel("LUXMC CLIENT PVP SUITE");
        titleLabel.setFont(new Font("SansSerif", Font.BOLD, 18));
        titleLabel.setForeground(new Color(56, 189, 248));

        JLabel subLabel = new JLabel("Menu In-Game · Clique para alternar módulos · Shift Direito ou ESC para fechar");
        subLabel.setFont(new Font("SansSerif", Font.PLAIN, 11));
        subLabel.setForeground(new Color(148, 163, 184));

        headerPanel.add(titleLabel, BorderLayout.NORTH);
        headerPanel.add(subLabel, BorderLayout.SOUTH);
        mainPanel.add(headerPanel, BorderLayout.NORTH);

        // Modules Grid
        JPanel gridPanel = new JPanel(new GridLayout(3, 3, 10, 10));
        gridPanel.setOpaque(false);
        gridPanel.setBorder(BorderFactory.createEmptyBorder(16, 0, 16, 0));

        gridPanel.add(createModuleToggle("FPS Display", modFps, new ActionListener() {
            @Override
            public void actionPerformed(ActionEvent e) {
                modFps = !modFps;
                updateButtonState((JButton) e.getSource(), modFps);
            }
        }));

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
        closeButton.setBackground(new Color(16, 185, 129));
        closeButton.setForeground(Color.WHITE);
        closeButton.setFocusPainted(false);
        closeButton.setBorder(BorderFactory.createEmptyBorder(10, 0, 10, 0));
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

        // ESC / Shift listener to close
        dialog.getRootPane().registerKeyboardAction(new ActionListener() {
            @Override
            public void actionPerformed(ActionEvent e) {
                dialog.setVisible(false);
                restoreMouse();
            }
        }, KeyStroke.getKeyStroke(KeyEvent.VK_ESCAPE, 0), JComponent.WHEN_IN_FOCUSED_WINDOW);

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
            button.setBackground(new Color(16, 185, 129, 220));
            button.setForeground(Color.WHITE);
            button.setBorder(BorderFactory.createLineBorder(new Color(52, 211, 153), 1));
        } else {
            button.setBackground(new Color(30, 41, 59, 200));
            button.setForeground(new Color(148, 163, 184));
            button.setBorder(BorderFactory.createLineBorder(new Color(71, 85, 105), 1));
        }
        String text = button.getText();
        if (text != null && !text.isEmpty()) {
            String base = text.replaceAll(" \\[ON\\]| \\[OFF\\]", "");
            button.setText(base + (active ? " [ON]" : " [OFF]"));
        }
    }

    // In-game HUD overlay window
    public static class InGameHudWindow extends JWindow {
        public InGameHudWindow() {
            setBackground(new Color(0, 0, 0, 0));
            setAlwaysOnTop(true);
            setFocusableWindowState(false);
            setSize(320, 240);
            setLocation(20, 20);

            javax.swing.Timer timer = new javax.swing.Timer(50, new ActionListener() {
                @Override
                public void actionPerformed(ActionEvent e) {
                    repaint();
                }
            });
            timer.start();
        }

        @Override
        public void paint(Graphics g) {
            super.paint(g);
            Graphics2D g2 = (Graphics2D) g.create();
            g2.setRenderingHint(RenderingHints.KEY_TEXT_ANTIALIASING, RenderingHints.VALUE_TEXT_ANTIALIAS_ON);

            int y = 20;

            if (modFps) {
                drawHudPill(g2, 10, y, "FPS: 144", new Color(16, 185, 129));
                y += 26;
            }

            if (modCps) {
                drawHudPill(g2, 10, y, "CPS: " + leftCps + " | " + rightCps, new Color(56, 189, 248));
                y += 26;
            }

            if (modCoords) {
                drawHudPill(g2, 10, y, "XYZ: 128, 64, -256", new Color(226, 184, 107));
                y += 26;
            }

            if (modPing) {
                drawHudPill(g2, 10, y, "Ping: 22ms", new Color(168, 85, 247));
                y += 26;
            }

            if (modSprint) {
                drawHudPill(g2, 10, y, "[Sprinting]", new Color(244, 63, 94));
                y += 26;
            }

            if (modKeystrokes) {
                drawKeystrokes(g2, 10, y);
            }

            g2.dispose();
        }

        private void drawHudPill(Graphics2D g2, int x, int y, String text, Color accent) {
            g2.setColor(new Color(11, 15, 25, 190));
            g2.fillRoundRect(x, y, 140, 22, 10, 10);
            g2.setColor(new Color(accent.getRed(), accent.getGreen(), accent.getBlue(), 120));
            g2.drawRoundRect(x, y, 140, 22, 10, 10);

            g2.setColor(accent);
            g2.setFont(new Font("Monospaced", Font.BOLD, 12));
            g2.drawString(text, x + 8, y + 16);
        }

        private void drawKeystrokes(Graphics2D g2, int x, int y) {
            // W
            drawKeyBox(g2, x + 24, y, "W", keyW);
            // A, S, D
            drawKeyBox(g2, x, y + 24, "A", keyA);
            drawKeyBox(g2, x + 24, y + 24, "S", keyS);
            drawKeyBox(g2, x + 48, y + 24, "D", keyD);
        }

        private void drawKeyBox(Graphics2D g2, int x, int y, String key, boolean pressed) {
            if (pressed) {
                g2.setColor(new Color(59, 130, 246, 220));
            } else {
                g2.setColor(new Color(15, 23, 42, 180));
            }
            g2.fillRoundRect(x, y, 22, 22, 6, 6);
            g2.setColor(new Color(255, 255, 255, pressed ? 255 : 120));
            g2.setFont(new Font("SansSerif", Font.BOLD, 11));
            g2.drawString(key, x + 6, y + 15);
        }
    }
}
