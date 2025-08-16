package detection;

public class CRC32 {
    public static class Result {
        public final String crc32;
        public final String codeword;
        public Result(String crc32, String codeword) {
            this.crc32 = crc32;
            this.codeword = codeword;
        }
    }
    private static int[] parseBits(String s) {
        int[] out = new int[s.length()];
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            if (c != '0' && c != '1')
                throw new IllegalArgumentException("Solo se aceptan '0' y '1'");
            out[i] = (c == '1') ? 1 : 0;
        }
        return out;
    }
    private static String bitsToString(int[] bits) {
        StringBuilder sb = new StringBuilder(bits.length);
        for (int b : bits) sb.append(b == 1 ? '1' : '0');
        return sb.toString();
    }
    private static int[] crc32PolyBits() {
        long poly = 0x04C11DB7L;
        int[] v = new int[33];
        v[0] = 1; // x^32
        for (int i = 31; i >= 0; i--) {
            v[32 - i] = (int)((poly >> i) & 1L);
        }
        return v;
    }
    private static int[] mod2DivideAligned(int[] dividend, int[] divisor) {
        int n = dividend.length, m = divisor.length;
        int[] work = dividend.clone();

        int i = 0;
        while (i <= n - m) {
            if (work[i] == 1) {
                for (int j = 0; j < m; j++) {
                    work[i + j] ^= divisor[j];
                }
            }
            i++;
            while (i <= n - m && work[i] == 0) i++;
        }
        int[] rem = new int[m - 1];
        System.arraycopy(work, n - (m - 1), rem, 0, m - 1);
        return rem; 
    }
    public static Result computePure(String messageBits) {
        int[] msg = parseBits(messageBits);
        int[] divisor = crc32PolyBits();

        int[] dividend = new int[msg.length + 32];
        System.arraycopy(msg, 0, dividend, 0, msg.length);

        int[] rem = mod2DivideAligned(dividend, divisor);

        String crc = bitsToString(rem);          
        String codeword = messageBits + crc;      
        return new Result(crc, codeword);
    }
    public static String generateCRC32Poly(String messageBits) {
        return computePure(messageBits).codeword;
    }
}