package com.luxmc.bootstrap;

import java.awt.BorderLayout;
import java.awt.Color;
import java.awt.Dimension;
import java.awt.Font;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.RenderingHints;
import java.io.BufferedInputStream;
import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URI;
import java.net.URL;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.nio.file.StandardCopyOption;
import javax.swing.BorderFactory;
import javax.swing.JButton;
import javax.swing.JFrame;
import javax.swing.JLabel;
import javax.swing.JPanel;
import javax.swing.JProgressBar;
import javax.swing.SwingConstants;
import javax.swing.SwingUtilities;
import javax.swing.UIManager;

public class Main {
    private static final String VERSION = "1.3.0-BETA";
    private static final String GITHUB_REPO = "predabr/luxmc";

    private static JFrame frame;
    private static JLabel statusLabel;
    private static JProgressBar progressBar;
    private static JButton launchButton;

    public static void main(String[] args) {
        boolean headless = false;
        try {
            if (java.awt.GraphicsEnvironment.isHeadless()) {
                headless = true;
            }
        } catch (Throwable t) {
            headless = true;
        }

        for (String arg : args) {
            if ("--headless".equalsIgnoreCase(arg) || "--cli".equalsIgnoreCase(arg)) {
                headless = true;
            }
        }

        if (headless) {
            runCli(args);
        } else {
            SwingUtilities.invokeLater(() -> createAndShowGUI(args));
        }
    }

    private static void createAndShowGUI(String[] args) {
        try {
            UIManager.setLookAndFeel(UIManager.getCrossPlatformLookAndFeelClassName());
        } catch (Exception ignored) {}

        frame = new JFrame("Luxmc Launcher " + VERSION);
        frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
        frame.setSize(480, 260);
        frame.setLocationRelativeTo(null);
        frame.setResizable(false);

        JPanel mainPanel = new JPanel() {
            @Override
            protected void paintComponent(Graphics g) {
                super.paintComponent(g);
                Graphics2D g2 = (Graphics2D) g.create();
                g2.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
                g2.setColor(new Color(12, 12, 14));
                g2.fillRect(0, 0, getWidth(), getHeight());
                g2.setColor(new Color(235, 208, 149, 40));
                g2.drawRoundRect(1, 1, getWidth() - 3, getHeight() - 3, 16, 16);
                g2.dispose();
            }
        };
        mainPanel.setLayout(new BorderLayout(16, 16));
        mainPanel.setBorder(BorderFactory.createEmptyBorder(24, 24, 24, 24));

        JLabel titleLabel = new JLabel("LUXMC LAUNCHER", SwingConstants.CENTER);
        titleLabel.setFont(new Font("SansSerif", Font.BOLD, 22));
        titleLabel.setForeground(new Color(235, 208, 149));

        JLabel versionLabel = new JLabel("v" + VERSION + " · Universal Bootstrap & Native Loader", SwingConstants.CENTER);
        versionLabel.setFont(new Font("SansSerif", Font.PLAIN, 12));
        versionLabel.setForeground(new Color(160, 160, 170));

        JPanel headerPanel = new JPanel(new BorderLayout(4, 4));
        headerPanel.setOpaque(false);
        headerPanel.add(titleLabel, BorderLayout.NORTH);
        headerPanel.add(versionLabel, BorderLayout.SOUTH);

        statusLabel = new JLabel("Inicializando ambiente nativo...", SwingConstants.CENTER);
        statusLabel.setFont(new Font("SansSerif", Font.PLAIN, 13));
        statusLabel.setForeground(new Color(220, 220, 230));

        progressBar = new JProgressBar(0, 100);
        progressBar.setValue(0);
        progressBar.setStringPainted(true);
        progressBar.setForeground(new Color(235, 208, 149));
        progressBar.setBackground(new Color(28, 29, 34));
        progressBar.setBorder(BorderFactory.createLineBorder(new Color(255, 255, 255, 25), 1));
        progressBar.setPreferredSize(new Dimension(420, 24));

        JPanel centerPanel = new JPanel(new BorderLayout(8, 8));
        centerPanel.setOpaque(false);
        centerPanel.add(statusLabel, BorderLayout.NORTH);
        centerPanel.add(progressBar, BorderLayout.SOUTH);

        launchButton = new JButton("INICIAR LUXMC");
        launchButton.setFont(new Font("SansSerif", Font.BOLD, 13));
        launchButton.setForeground(new Color(12, 12, 14));
        launchButton.setBackground(new Color(235, 208, 149));
        launchButton.setFocusPainted(false);
        launchButton.setBorder(BorderFactory.createEmptyBorder(10, 16, 10, 16));
        launchButton.setEnabled(false);

        mainPanel.add(headerPanel, BorderLayout.NORTH);
        mainPanel.add(centerPanel, BorderLayout.CENTER);
        mainPanel.add(launchButton, BorderLayout.SOUTH);

        frame.setContentPane(mainPanel);
        frame.setVisible(true);

        new Thread(() -> processBootstrap(args)).start();
    }

    private static void updateStatus(String text, int progress) {
        SwingUtilities.invokeLater(() -> {
            if (statusLabel != null) statusLabel.setText(text);
            if (progressBar != null) {
                progressBar.setValue(progress);
                progressBar.setString(progress + "%");
            }
        });
    }

    private static void runCli(String[] args) {
        System.out.println("==================================================");
        System.out.println("  Luxmc Launcher v" + VERSION + " - Universal Bootstrap");
        System.out.println("==================================================");
        processBootstrap(args);
    }

    private static void processBootstrap(String[] args) {
        try {
            updateStatus("Identificando sistema operacional...", 15);
            String os = System.getProperty("os.name", "").toLowerCase();
            String arch = System.getProperty("os.arch", "").toLowerCase();
            boolean isWindows = os.contains("win");
            boolean isMac = os.contains("mac");
            boolean isLinux = !isWindows && !isMac;

            updateStatus("Verificando binário local do Luxmc...", 35);
            File nativeExecutable = findLocalBinary(isWindows, isMac);

            if (nativeExecutable != null && nativeExecutable.exists() && nativeExecutable.canExecute()) {
                updateStatus("Iniciando Luxmc nativo: " + nativeExecutable.getName(), 100);
                launchNative(nativeExecutable, args);
                return;
            }

            File appDir = getAppDirectory(isWindows, isMac);
            if (!appDir.exists()) {
                appDir.mkdirs();
            }

            File targetBinary = new File(appDir, isWindows ? "Luxmc.exe" : "luxmc");

            if (targetBinary.exists() && targetBinary.canExecute()) {
                updateStatus("Iniciando Luxmc instalado...", 100);
                launchNative(targetBinary, args);
                return;
            }

            updateStatus("Buscando release oficial no GitHub...", 50);
            boolean downloaded = downloadLatestRelease(targetBinary, isWindows, isMac);

            if (downloaded && targetBinary.exists()) {
                if (!isWindows) {
                    targetBinary.setExecutable(true);
                }
                updateStatus("Iniciando Luxmc v" + VERSION + "...", 100);
                launchNative(targetBinary, args);
            } else {
                updateStatus("Execute o instalador oficial ou rode 'luxmc' no terminal.", 100);
                if (launchButton != null) {
                    SwingUtilities.invokeLater(() -> {
                        launchButton.setEnabled(true);
                        launchButton.setText("FECHAR");
                        launchButton.addActionListener(e -> System.exit(0));
                    });
                }
            }
        } catch (Exception e) {
            updateStatus("Erro no bootstrap: " + e.getMessage(), 100);
            e.printStackTrace();
        }
    }

    private static File findLocalBinary(boolean isWindows, boolean isMac) {
        String exeName = isWindows ? "luxmc.exe" : "luxmc";
        String capitalizedExe = isWindows ? "Luxmc.exe" : "Luxmc";

        File[] candidates = new File[] {
            new File(exeName),
            new File(capitalizedExe),
            new File("src-tauri/target/release/" + exeName),
            new File("src-tauri/target/debug/" + exeName),
            new File("../src-tauri/target/release/" + exeName),
            new File("../src-tauri/target/debug/" + exeName),
            new File(isWindows ? "C:\\Program Files\\Luxmc\\Luxmc.exe" : "/usr/bin/luxmc"),
            new File(isWindows ? System.getenv("LOCALAPPDATA") + "\\Programs\\Luxmc\\Luxmc.exe" : System.getProperty("user.home") + "/.local/bin/luxmc")
        };

        for (File candidate : candidates) {
            if (candidate != null && candidate.exists()) {
                if (!isWindows) candidate.setExecutable(true);
                return candidate;
            }
        }
        return null;
    }

    private static File getAppDirectory(boolean isWindows, boolean isMac) {
        String userHome = System.getProperty("user.home");
        if (isWindows) {
            String appData = System.getenv("APPDATA");
            return new File(appData != null ? appData : userHome, "Luxmc/bin");
        } else if (isMac) {
            return new File(userHome, "Library/Application Support/Luxmc/bin");
        } else {
            return new File(userHome, ".local/share/luxmc/bin");
        }
    }

    private static boolean downloadLatestRelease(File targetFile, boolean isWindows, boolean isMac) {
        try {
            String assetPattern = isWindows ? ".exe" : (isMac ? ".dmg" : "appimage");
            String apiUrl = "https://api.github.com/repos/" + GITHUB_REPO + "/releases/latest";
            
            updateStatus("Conectando ao GitHub Releases...", 60);
            HttpURLConnection conn = (HttpURLConnection) URI.create(apiUrl).toURL().openConnection();
            conn.setRequestProperty("User-Agent", "Luxmc-Bootstrap/" + VERSION);
            conn.setRequestProperty("Accept", "application/vnd.github.v3+json");
            conn.setConnectTimeout(8000);
            conn.setReadTimeout(12000);

            if (conn.getResponseCode() != 200) {
                updateStatus("Repositório atualizado. Verifique: github.com/" + GITHUB_REPO, 100);
                return false;
            }

            InputStream in = conn.getInputStream();
            String json = new String(in.readAllBytes(), "UTF-8");
            in.close();

            String downloadUrl = null;
            int idx = 0;
            while ((idx = json.indexOf("\"browser_download_url\":", idx)) != -1) {
                int start = json.indexOf("\"", idx + 23) + 1;
                int end = json.indexOf("\"", start);
                String url = json.substring(start, end);
                if (url.toLowerCase().contains(assetPattern)) {
                    downloadUrl = url;
                    break;
                }
                idx = end;
            }

            if (downloadUrl == null) {
                return false;
            }

            updateStatus("Baixando Luxmc atualizado...", 70);
            HttpURLConnection dlConn = (HttpURLConnection) URI.create(downloadUrl).toURL().openConnection();
            dlConn.setRequestProperty("User-Agent", "Luxmc-Bootstrap/" + VERSION);
            dlConn.setConnectTimeout(10000);
            long totalBytes = dlConn.getContentLengthLong();

            try (BufferedInputStream bin = new BufferedInputStream(dlConn.getInputStream());
                 FileOutputStream fos = new FileOutputStream(targetFile)) {
                byte[] buf = new byte[8192];
                long downloaded = 0;
                int read;
                while ((read = bin.read(buf)) != -1) {
                    fos.write(buf, 0, read);
                    downloaded += read;
                    if (totalBytes > 0) {
                        int pct = 70 + (int) ((downloaded * 28) / totalBytes);
                        updateStatus("Baixando: " + (downloaded / (1024 * 1024)) + " MB...", pct);
                    }
                }
            }

            return true;
        } catch (Exception e) {
            System.err.println("Download failed: " + e.getMessage());
            return false;
        }
    }

    private static void launchNative(File binary, String[] args) {
        try {
            ProcessBuilder pb = new ProcessBuilder();
            String[] cmd = new String[args.length + 1];
            cmd[0] = binary.getAbsolutePath();
            System.arraycopy(args, 0, cmd, 1, args.length);
            pb.command(cmd);
            pb.inheritIO();
            pb.start();
            Thread.sleep(1200);
            System.exit(0);
        } catch (Exception e) {
            System.err.println("Erro ao executar binário nativo: " + e.getMessage());
            updateStatus("Falha ao abrir processo: " + e.getMessage(), 100);
        }
    }
}
