import java.net.DatagramPacket;
import java.net.InetAddress;
import java.net.MulticastSocket;
import java.net.Socket;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;

public final class LanListProbe {
    public static void main(String[] args) throws Exception {
        try (MulticastSocket discovery = new MulticastSocket(4445)) {
            discovery.joinGroup(InetAddress.getByName("224.0.2.60"));
            discovery.setSoTimeout(30000);
            System.out.println("READY");
            long deadline = System.currentTimeMillis() + 30000;
            while (System.currentTimeMillis() < deadline) {
                DatagramPacket packet = new DatagramPacket(new byte[1024], 1024);
                discovery.receive(packet);
                String message = new String(packet.getData(), 0, packet.getLength(), StandardCharsets.UTF_8);
                if (!message.contains("[Luxmc]") || !message.contains("Mundo de amigo")) continue;
                int start = message.indexOf("[AD]") + 4;
                int end = message.indexOf("[/AD]");
                if (start < 4 || end < start) continue;
                int port = Integer.parseInt(message.substring(start, end));
                try (Socket game = new Socket(packet.getAddress(), port)) {
                    game.setSoTimeout(5000);
                    byte[] expected = "minecraft lan list".getBytes(StandardCharsets.UTF_8);
                    game.getOutputStream().write(expected);
                    byte[] response = new byte[expected.length];
                    int offset = 0;
                    while (offset < response.length) {
                        int count = game.getInputStream().read(response, offset, response.length - offset);
                        if (count < 0) throw new IllegalStateException("Proxy closed before response");
                        offset += count;
                    }
                    if (!Arrays.equals(expected, response)) throw new IllegalStateException("Unexpected game response");
                    System.out.println("JAVA_LAN_DISCOVERY_AND_GAME_TRANSFER_OK");
                    return;
                }
            }
            throw new IllegalStateException("World not found in Java LAN discovery");
        }
    }
}
