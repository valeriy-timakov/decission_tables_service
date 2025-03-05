package net.home.decision_tables_service;



public class DecisionTablesService {

    static {
        System.loadLibrary("decision_tables_service");
        //System.load("C:\\Users\\valti\\Projects\\RustroverProjects\\decision_tables_service\\target\\release\\decision_tables_service.dll");

    }

    public native int createDecisionTable(String dt_file_path, int data_start_idx);
    public native String[] executeRequest(int dt_index, String request);


}