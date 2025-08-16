package app;

import correction.Hamming;
import detection.CRC32;

import java.io.OutputStreamWriter;
import java.io.PrintWriter;
import java.net.Socket;
import java.util.Random;

public class Bench {

    private static String asciiToBits(String s) {
        StringBuilder out = new StringBuilder();
        for (int i = 0; i < s.length(); i++) {
            int v = s.charAt(i) & 0xFF;
            String b = String.format("%8s", Integer.toBinaryString(v)).replace(' ', '0');
            out.append(b);
        }
        return out.toString();
    }

    private static String applyNoise(String bits, double p) {
        Random rnd = new Random();
        StringBuilder sb = new StringBuilder(bits.length());
        for (int i = 0; i < bits.length(); i++) {
            char c = bits.charAt(i);
            boolean flip = rnd.nextDouble() < p;
            sb.append(flip ? (c == '1' ? '0' : '1') : c);
        }
        return sb.toString();
    }

    private static void sendToReceiver(String host, int port, String algo, String param, String bits) throws Exception {
        try (Socket sock = new Socket(host, port);
             PrintWriter out = new PrintWriter(new OutputStreamWriter(sock.getOutputStream()), true)) {
            out.println("ALGO=" + algo);
            out.println("PARAM=" + param);
            out.println("BITS=" + bits);
        }
    }

    // Uso:
    // Hamming: java -cp out app.Bench <texto> 1 <n> <ber> <host> <port>
    // CRC32 :  java -cp out app.Bench <texto> 2 <ber> <host> <port>
    public static void main(String[] args) throws Exception {
        if (args.length < 5) {
            System.err.println("Uso:\n  Hamming: java -cp out app.Bench <texto> 1 <n> <ber> <host> <port>\n  CRC32 :  java -cp out app.Bench <texto> 2 <ber> <host> <port>");
            return;
        }

        String text = args[0];
        String algoSel = args[1];

        String dataBits = asciiToBits(text);
        String algo, param, frameBits;

        if ("1".equals(algoSel)) {
            if (args.length < 6) {
                System.err.println("Faltan args para Hamming.");
                return;
            }
            int n = Integer.parseInt(args[2]);
            double ber = Double.parseDouble(args[3]);
            String host = args[4];
            int port = Integer.parseInt(args[5]);

            Hamming.EncodeResult res = Hamming.encodeStream(dataBits, n);
            algo = "HAMMING";
            param = "n=" + n + ";pad=" + res.paddingZeros;
            frameBits = applyNoise(res.encodedBits, ber);
            sendToReceiver(host, port, algo, param, frameBits);
            System.out.println("OK Hamming -> bits=" + frameBits.length());

        } else if ("2".equals(algoSel)) {
            if (args.length < 5) {
                System.err.println("Faltan args para CRC32.");
                return;
            }
            double ber = Double.parseDouble(args[2]);
            String host = args[3];
            int port = Integer.parseInt(args[4]);

            CRC32.Result r = CRC32.computePure(dataBits);
            algo = "CRC32";
            param = "mode=PURE";
            frameBits = applyNoise(r.codeword, ber);
            sendToReceiver(host, port, algo, param, frameBits);
            System.out.println("OK CRC32 -> bits=" + frameBits.length());

        } else {
            System.err.println("Algoritmo desconocido: " + algoSel);
        }
    }
}
