package net.home.decision_tables_service;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.lang.management.ManagementFactory;

public class DecisionTablesServiceTest {

    @Test
    public void testCreateDecisionTable() {
        DecisionTablesService bridge = new DecisionTablesService();
        int idx = bridge.createDecisionTable("D:\\Downloads\\1\\Printouts.xlsx", 6);
        Assertions.assertEquals(0, idx);
        var request = """
            {
                "salesProgramCode": 250004,
                "acquisitionType": "NEW",
                "creditContractYear": "Default",
                "signatory": "AGENT",
                "signingOption": "ELECTRONIC",
                "salesman": "AGENT_NETWORK"
            }""";
        var response = bridge.executeRequest(0, request);
        Assertions.assertNotNull(response);
        Assertions.assertEquals(1, response.length);
        Assertions.assertEquals("{\"Preamble\": \"empty\", \"SigningPerson\": \"empty\", \"Text\": \"empty\", \"Note\": \"empty\", \"Statement\": \"empty\", \"Notification\": \"some_text_notification_template\"}", response[0]);
    }



    public static void main(String[] args) throws IOException {
        var request = """
             {
                "salesProgramCode": 10368,
                "acquisitionType": "RENEWAL_FROM_OTHERS",
                "creditContractYear": "Default",
                "signatory": "AGENT",
                "signingOption": "ELECTRONIC",
                "salesman": "AGENT"
        }""";
        var console = new BufferedReader(new InputStreamReader(System.in));

        waitForEnter(console, "Press Enter to load the decision table...");
        DecisionTablesService bridge = new DecisionTablesService();
        long start = System.currentTimeMillis();
        int idx = bridge.createDecisionTable("EnterPrintouts_FullRights_e766157e-e804-4a73-8dc2-8074242c2b74.xlsx", 6);
        System.out.println("Load time: " + (System.currentTimeMillis() - start));
        System.out.println("idx: " + idx); // 7

        waitForEnter(console, "Press Enter to execute the request...");
        try {
            start = System.currentTimeMillis();
            var response = bridge.executeRequest(idx, request);
            System.out.println("Execute time: " + (System.currentTimeMillis() - start));
            for (var s : response) {
                System.out.println(s);
            }
        } catch (Exception e) {
            System.out.println(e.getMessage());
        }

        waitForEnter(console, "Press Enter to exit...");
    }

    private static void waitForEnter(BufferedReader console, String prompt) throws IOException {
        printMemoryUsage();
        System.out.println(prompt);
        console.readLine();
    }

    private static void printMemoryUsage() {
        var runtime = Runtime.getRuntime();
        var memory = ManagementFactory.getMemoryMXBean();
        long heapUsed = runtime.totalMemory() - runtime.freeMemory();
        System.out.printf("Memory: heap used %s of %s, non-heap used %s, process RSS %s%n",
                formatBytes(heapUsed),
                formatBytes(runtime.totalMemory()),
                formatBytes(memory.getNonHeapMemoryUsage().getUsed()),
                readProcessRss());
    }

    private static String formatBytes(long bytes) {
        return String.format("%.1f MB", bytes / 1024.0 / 1024.0);
    }

    private static String readProcessRss() {
        try {
            var process = new ProcessBuilder("ps", "-o", "rss=", "-p", Long.toString(ProcessHandle.current().pid()))
                    .redirectErrorStream(true)
                    .start();
            String output;
            try (var reader = new BufferedReader(new InputStreamReader(process.getInputStream()))) {
                output = reader.readLine();
            }
            process.waitFor();
            return output == null || output.isBlank()
                    ? "n/a"
                    : formatBytes(Long.parseLong(output.trim()) * 1024L);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            return "n/a";
        } catch (IOException | NumberFormatException e) {
            return "n/a";
        }
    }
}

