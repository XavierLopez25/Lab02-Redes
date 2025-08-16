package app;

import java.io.*;
import java.net.Socket;
import java.util.Random;

import correction.Hamming;
import detection.CRC32;

public class Main {
    private static String readLine(String prompt) throws IOException {
        System.out.print(prompt);
        BufferedReader br = new BufferedReader(new InputStreamReader(System.in));
        return br.readLine();
    }

    // ==== APLICACIÓN / PRESENTACIÓN ====
    private static String asciiToBits(String s) {
        StringBuilder out = new StringBuilder();
        for (int i = 0; i < s.length(); i++) {
            int v = s.charAt(i) & 0xFF;
            String b = String.format("%8s", Integer.toBinaryString(v)).replace(' ', '0');
            out.append(b);
        }
        return out.toString();
    }

    // ==== RUIDO ====
    private static String applyNoise(String bits, double p) {
        Random rnd = new Random();
        StringBuilder sb = new StringBuilder(bits.length());
        for (int i = 0; i < bits.length(); i++) {
            char c = bits.charAt(i);
            boolean flip = rnd.nextDouble() < p;
            if (flip) sb.append(c == '1' ? '0' : '1'); else sb.append(c);
        }
        return sb.toString();
    }

    // ==== TRANSMISIÓN (cliente TCP) ====
    private static void sendToReceiver(String host, int port, String algo, String param, String bits) throws IOException {
        try (Socket sock = new Socket(host, port);
             PrintWriter out = new PrintWriter(new OutputStreamWriter(sock.getOutputStream()), true)) {
            out.println("ALGO=" + algo);
            out.println("PARAM=" + param);
            out.println("BITS=" + bits);
        }
    }

    public static void main(String[] args) throws Exception {
        System.out.println("=== EMISOR (Parte 2) ===");

        // APLICACIÓN
        String text = readLine("Texto a enviar: ");
        System.out.println("Algoritmos: 1) Hamming  2) CRC-32 (puro)");
        String choice = readLine("Selecciona algoritmo [1/2]: ");

        // PRESENTACIÓN
        String dataBits = asciiToBits(text);

        String algo, param, frameBits; // salida de ENLACE
        if ("1".equals(choice)) {
            algo = "HAMMING";
            int n = Integer.parseInt(readLine("Hamming: n del código (p.ej., 7): "));
            Hamming.EncodeResult res = Hamming.encodeStream(dataBits, n);
            param = "n=" + n + ";pad=" + res.paddingZeros;
            frameBits = res.encodedBits;
        } else if ("2".equals(choice)) {
            algo = "CRC32";
            CRC32.Result r = CRC32.computePure(dataBits);
            param = "mode=PURE";
            frameBits = r.codeword;
        } else {
            System.err.println("Opción inválida");
            return;
        }

        // RUIDO
        double p = 0.0; // BER por defecto sin ruido
        try {
            p = Double.parseDouble(readLine("Probabilidad de error por bit (p.ej., 0.01): "));
        } catch (Exception ignore) {}
        String noisy = applyNoise(frameBits, p);

        // TRANSMISIÓN
        String host = readLine("Host receptor [127.0.0.1]: ");
        if (host == null || host.isBlank()) host = "127.0.0.1";
        int port = 9000;
        try {
            String pStr = readLine("Puerto receptor [9000]: ");
            if (pStr != null && !pStr.isBlank()) port = Integer.parseInt(pStr.trim());
        } catch (Exception ignore) {}

        sendToReceiver(host, port, algo, param, noisy);
        System.out.println("Trama enviada. Longitud: " + noisy.length() + " bits");
    }
}