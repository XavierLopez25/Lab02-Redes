
package correction; 

public class Hamming {
    public static class EncodeResult {
        public final String encodedBits;
        public final int paddingZeros;
        public EncodeResult(String encodedBits, int paddingZeros) {
            this.encodedBits = encodedBits;
            this.paddingZeros = paddingZeros;
        }
    }
    private static boolean isPowerOfTwo(int x) {
        return x != 0 && (x & (x - 1)) == 0;
    }
    private static int parityBitsCount(int n) {
        int r = 0;
        while ((1 << r) < (n + 1)) r++;
        return r;
    }
    private static int[] parseBits(String s) {
        int[] out = new int[s.length()];
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            if (c != '0' && c != '1') throw new IllegalArgumentException("Solo se aceptan '0' y '1'");
            out[i] = (c == '1') ? 1 : 0;
        }
        return out;
    }
    private static String bitsToString(int[] bits) {
        StringBuilder sb = new StringBuilder(bits.length);
        for (int b : bits) sb.append(b == 1 ? '1' : '0');
        return sb.toString();
    }
    public static EncodeResult encodeStream(String dataBits, int n) {
        int[] data = parseBits(dataBits);
        int r = parityBitsCount(n);
        int m = n - r;
        if (m <= 0) throw new IllegalArgumentException("n demasiado pequeño para código Hamming");

        int padding = (m - (data.length % m)) % m;
        int totalData = data.length + padding;

        int blocks = totalData / m;
        int[] dataPadded = new int[totalData];
        System.arraycopy(data, 0, dataPadded, 0, data.length);

        StringBuilder out = new StringBuilder(blocks * n);
        int idx = 0; 

        for (int b = 0; b < blocks; b++) {
            int[] codeword = new int[n];

            int dp = 0;
            for (int pos = 1; pos <= n; pos++) {
                if (!isPowerOfTwo(pos)) {
                    codeword[pos - 1] = dataPadded[idx + dp];
                    dp++;
                    if (dp == m) break;
                }
            }

            for (int i = 0; i < r; i++) {
                int p = 1 << i; 
                int parity = 0;
                for (int pos = 1; pos <= n; pos++) {
                    if ((pos & p) != 0) {
                        parity ^= codeword[pos - 1];
                    }
                }
                codeword[p - 1] = parity;
            }

            out.append(bitsToString(codeword));
            idx += m;
        }

        return new EncodeResult(out.toString(), padding);
    }
}
