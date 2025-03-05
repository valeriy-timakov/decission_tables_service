package net.home.decision_tables_service;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;

public class DecisionTablesServiceTest {

    @Test
    public void testCreateDecisionTable() {
        DecisionTablesService bridge = new DecisionTablesService();
        int idx = bridge.createDecisionTable("D:\\Downloads\\1\\EnterPrintouts_FullRights.xlsx", 6);
        Assertions.assertEquals(0, idx);
        var request = """
            {
                "salesProgramCode": 600004,
                "acquisitionType": "NEW",
                "creditContractYear": "Default",
                "signatory": "AGENT",
                "signingOption": "ELECTRONIC",
                "salesman": "AGENT_NETWORK"
            }""";
        var response = bridge.executeRequest(0, request);
        Assertions.assertNotNull(response);
        Assertions.assertEquals(1, response.length);
        Assertions.assertEquals("{\"Preamble\": \"empty\", \"SigningPerson\": \"empty\", \"Text\": \"empty\", \"Note\": \"empty\", \"Statement\": \"empty\", \"Notification\": \"casco_ARX_digital_notification_0001\"}", response[0]);
    }



    public static void main(String[] args) {
        var request = """
             {
            "salesProgramCode": 600004,
                "acquisitionType": "NEW",
                "creditContractYear": "Default",
                "signatory": "AGENT",
                "signingOption": "ELECTRONIC",
                "salesman": "AGENT_NETWORK"
        }""";
        DecisionTablesService bridge = new DecisionTablesService();
        long start = System.currentTimeMillis();
        int idx = bridge.createDecisionTable("D:\\Downloads\\1\\EnterPrintouts_FullRights.xlsx", 6);
        System.out.println("Load time: " + (System.currentTimeMillis() - start));
        System.out.println("idx: " + idx); // 7
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
    }
}

