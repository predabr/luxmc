package io.github.luxmc.client;

public final class ClassFileHints {
    public static final class Hint {
        private final byte[] value;
        private final int[] skips = new int[256];
        public Hint(String text) {
            value = text.getBytes(java.nio.charset.StandardCharsets.US_ASCII);
            if (value.length == 0) throw new IllegalArgumentException("Empty hint");
            java.util.Arrays.fill(skips, value.length);
            for (int index = 0; index < value.length - 1; index++) skips[value[index] & 255] = value.length - index - 1;
        }
    }

    public static boolean contains(byte[] bytes, Hint hint) {
        if (bytes == null || bytes.length < 10 || bytes[0] != (byte)0xca || bytes[1] != (byte)0xfe || bytes[2] != (byte)0xba || bytes[3] != (byte)0xbe) return false;
        byte[] needle = hint.value;
        int count = ((bytes[8] & 255) << 8) | (bytes[9] & 255);
        int offset = 10;
        for (int index = 1; index < count; index++) {
            if (offset >= bytes.length) return false;
            int tag = bytes[offset++] & 255;
            int length;
            switch (tag) {
                case 1:
                    if (offset + 2 > bytes.length) return false;
                    length = ((bytes[offset] & 255) << 8) | (bytes[offset + 1] & 255);
                    offset += 2;
                    if (offset + length > bytes.length) return false;
                    for (int start = offset; start <= offset + length - needle.length;) {
                        int match = needle.length - 1;
                        while (match >= 0 && bytes[start + match] == needle[match]) match--;
                        if (match < 0) return true;
                        start += hint.skips[bytes[start + needle.length - 1] & 255];
                    }
                    break;
                case 3: case 4: case 9: case 10: case 11: case 12: case 17: case 18: length = 4; break;
                case 5: case 6: length = 8; index++; break;
                case 7: case 8: case 16: case 19: case 20: length = 2; break;
                case 15: length = 3; break;
                default: return false;
            }
            offset += length;
            if (offset > bytes.length) return false;
        }
        return false;
    }
}
