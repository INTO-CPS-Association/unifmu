from fmpy import read_model_description, extract
from fmpy.fmi2 import FMU2Slave, fmi2OK
import shutil
import sys


if __name__ == "__main__":
    if len(sys.argv) > 1:
        fmu_filename = str(sys.argv[1])
    else: 
        fmu_filename = "myfmu2.fmu"

    start_time = 0.0
    sim_time = start_time
    step_size = 1e-2
    
    # read the model description
    model_description = read_model_description(fmu_filename)
    can_handle_state = model_description.coSimulation.canGetAndSetFMUstate
    
    if can_handle_state:
        print("FMU can get and set state")
    
    
    print("Starting test on FMU: ", fmu_filename)
            
    # collect the value references
    vrs = {}
    for variable in model_description.modelVariables:
        vrs[variable.name] = variable.valueReference

    # extract the FMU
    unzipdir = extract(fmu_filename)

    fmu = FMU2Slave(guid=model_description.guid,
                    unzipDirectory=unzipdir,
                    modelIdentifier=model_description.coSimulation.modelIdentifier,
                    instanceName='instance1')
    
    # Checking platform and version
    # assert fmu.getTypesPlatform() == "default"
    # assert fmu.getVersion() == "2.0"  

    # initialize
    fmu.instantiate()
    fmu.setupExperiment(startTime=start_time)
    fmu.enterInitializationMode()
    fmu.exitInitializationMode()

    # Fetching initial values
    print("Fetching initial values from the FMU")

    real_a = fmu.getReal([vrs["real_a"]])[0]
    real_b = fmu.getReal([vrs["real_b"]])[0]
    real_c = fmu.getReal([vrs["real_c"]])[0]
    
    integer_a = fmu.getInteger([vrs["integer_a"]])[0]
    integer_b = fmu.getInteger([vrs["integer_b"]])[0]
    integer_c = fmu.getInteger([vrs["integer_c"]])[0]
    
    boolean_a = fmu.getBoolean([vrs["boolean_a"]])[0]
    boolean_b = fmu.getBoolean([vrs["boolean_b"]])[0]
    booler_c = fmu.getBoolean([vrs["boolean_c"]])[0]
    
    string_a = fmu.getString([vrs["string_a"]])[0].decode("utf-8")
    string_b = fmu.getString([vrs["string_b"]])[0].decode("utf-8")
    string_c = fmu.getString([vrs["string_c"]])[0].decode("utf-8")
    
    print("Asserting initial values")
    
    assert real_a == 0.0
    assert real_b == 0.0    
    assert real_c == 0.0
    assert integer_a == 0
    assert integer_b == 0
    assert integer_c == 0
    assert boolean_a == False
    assert boolean_b == False
    assert booler_c == False
    assert string_a == ""
    assert string_b == ""
    assert string_c == ""
    
    if can_handle_state:
        print("Fetching FMU state")
        state = fmu.getFMUstate()
        rerolled_time = sim_time
        print("from test_fmi2.py: The state is: ", state)

    print(f"Updating inputs at time {sim_time}")
    fmu.setReal([vrs["real_a"],vrs["real_b"]],[1.0,2.0])
    fmu.setInteger([vrs["integer_a"],vrs["integer_b"]],[1,2])
    fmu.setBoolean([vrs["boolean_a"],vrs["boolean_b"]],[True,False])
    fmu.setString([vrs["string_a"],vrs["string_b"]],["Hello, ","World!"])

    print(f"Doing a step of size {step_size} at time {sim_time}")
    fmu.doStep(sim_time, step_size) 
    sim_time += step_size
    

    print("Fetching values")
    real_c = fmu.getReal([vrs["real_c"]])[0]
    integer_c = fmu.getInteger([vrs["integer_c"]])[0]
    boolean_c = fmu.getBoolean([vrs["boolean_c"]])[0]
    string_c = fmu.getString([vrs["string_c"]])[0].decode("utf-8")
    
    print("Asserting values")
    assert real_c == 3.0
    assert integer_c == 3
    assert boolean_c == True
    assert string_c == "Hello, World!"
    
    if can_handle_state: 
        print("Rerolling FMU state")
        fmu.setFMUstate(state)
        print("Resetting time")
        sim_time = rerolled_time
                
        print("Fetching rerolled values")
        real_a = fmu.getReal([vrs["real_a"]])[0]
        real_b = fmu.getReal([vrs["real_b"]])[0]
        real_c = fmu.getReal([vrs["real_c"]])[0]

        integer_a = fmu.getInteger([vrs["integer_a"]])[0]
        integer_b = fmu.getInteger([vrs["integer_b"]])[0]
        integer_c = fmu.getInteger([vrs["integer_c"]])[0]

        boolean_a = fmu.getBoolean([vrs["boolean_a"]])[0]
        boolean_b = fmu.getBoolean([vrs["boolean_b"]])[0]
        booler_c = fmu.getBoolean([vrs["boolean_c"]])[0]

        string_a = fmu.getString([vrs["string_a"]])[0].decode("utf-8")
        string_b = fmu.getString([vrs["string_b"]])[0].decode("utf-8")
        string_c = fmu.getString([vrs["string_c"]])[0].decode("utf-8")

        print("Asserting rerolled values")

        assert real_a == 0.0
        assert real_b == 0.0    
        assert real_c == 0.0
        assert integer_a == 0
        assert integer_b == 0
        assert integer_c == 0
        assert boolean_a == False
        assert boolean_b == False
        assert booler_c == False
        assert string_a == ""
        assert string_b == ""
        assert string_c == ""
        
    print("Tests passed")
    print("Terminating")
   
    fmu.terminate()
    fmu.freeInstance()
    
    shutil.rmtree(unzipdir, ignore_errors=True)
    
    print("Test completed") 