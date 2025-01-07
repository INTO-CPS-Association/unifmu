from fmpy import read_model_description, extract
from fmpy.fmi3 import FMU3Slave
import shutil
import os
import sys

if __name__ == "__main__":
    if len(sys.argv) > 1:
        fmu_filename = str(sys.argv[1])
    else:
        #fmu_filename = "test1417.fmu" 
        fmu_filename = "myfmu3.fmu"

    start_time = 0.0
    sim_time = start_time
    threshold = 2.0
    step_size = 1e-2
    
    
    # Read the model description
    model_description = read_model_description(fmu_filename)
    can_handle_state = model_description.coSimulation.canGetAndSetFMUstate
    
    if can_handle_state:
        print("FMU can get and set state")
        
     
    # Collect valuereferences
    vrs = {}
    for variable in model_description.modelVariables:
        vrs[variable.name] = variable.valueReference
        
    # Extract the FMU
    unzipdir = extract(fmu_filename)
    #unzipdir = os.path.abspath(unzipdir)
    
    fmu = FMU3Slave(guid=model_description.guid,
                         unzipDirectory=unzipdir,
                         modelIdentifier=model_description.coSimulation.modelIdentifier,
                         instanceName='instance1')
    
    assert fmu.getVersion() == "3.0", "FMU is not version 3.0"
     
    # Initialize FMU
    print("Initializing FMI3 test")
    fmu.instantiate()  # This one prints the "Reading configuration file" message
    fmu.enterInitializationMode()
    fmu.exitInitializationMode()
    # Fetching initial values
    print("Fetching initial values from the FMU")
    
    float32 = fmu.getFloat32([vrs["float32_a"], vrs["float32_b"], vrs["float32_c"]])
    float64 = fmu.getFloat64([vrs["float64_a"], vrs["float64_b"], vrs["float64_a"]])

    int8 = fmu.getInt8([vrs["int8_a"], vrs["int8_b"], vrs["int8_c"]])
    uint8 = fmu.getUInt8([vrs["uint8_a"], vrs["uint8_b"], vrs["uint8_c"]])
    int16 = fmu.getInt16([vrs["int16_a"], vrs["int16_b"], vrs["int16_c"]])
    uint16 = fmu.getUInt16([vrs["uint16_a"], vrs["uint16_b"], vrs["uint16_c"]])
    int32 = fmu.getInt32([vrs["int32_a"], vrs["int32_b"], vrs["int32_c"]])
    uint32 = fmu.getUInt32([vrs["uint32_a"], vrs["uint32_b"], vrs["uint32_c"]])
    int64 = fmu.getInt64([vrs["int64_a"], vrs["int64_b"], vrs["int64_c"]])
    uint64 = fmu.getUInt64([vrs["uint64_a"], vrs["uint64_b"], vrs["uint64_c"]])

    boolean = fmu.getBoolean([vrs["boolean_a"], vrs["boolean_b"], vrs["boolean_c"]])
    string = fmu.getString([vrs["string_a"], vrs["string_b"], vrs["string_c"]])
    
    #binary = fmu.getBinary([vrs["binary_a"], vrs["binary_b"], vrs["binary_c"]])
    
    #clock = fmu.getClock([vrs["clock_a"], vrs["clock_b"], vrs["clock_c"]])
    
    print("Asserting initial values")
    
    assert float32 == [0.0, 0.0, 0.0]
    assert float64 == [0.0, 0.0, 0.0]
    
    assert int8 == [0, 0, 0]
    assert uint8 == [0, 0, 0]
    assert int16 == [0, 0, 0]
    assert uint16 == [0, 0, 0]
    assert int32 == [0, 0, 0]
    assert uint32 == [0, 0, 0]
    assert int64 == [0, 0, 0]
    assert uint64 == [0, 0, 0]
    assert boolean == [False, False, False]
    
    assert string == ["", "", ""]
    
    #assert binary_a == b""
    #assert binary_b == b""
    #assert binary_c == b""
    #assert clock_a == 0
    #assert clock_b == 0
    #assert clock_c == 0
    
    # Saving initial state   
    #print("Saving initial state") 
    #fmu_state = fmu.getFMUState()
    #print(type(fmu_state))
    #fmu_state = fmu.serializeFMUState(fmu_state)
    #print(fmu_state)
    
    # Simulating
    print(f"Updating inputs at time {sim_time}")
    
    fmu.setFloat32([vrs["float32_a"],vrs["float32_b"]],[1.0,2.0])
    fmu.setFloat64([vrs["float64_a"],vrs["float64_b"]],[1.0,2.0])
    fmu.setInt8([vrs["int8_a"],vrs["int8_b"]],[1,2])
    fmu.setUInt8([vrs["uint8_a"],vrs["uint8_b"]],[1,2])
    fmu.setInt16([vrs["int16_a"],vrs["int16_b"]],[1,2])
    fmu.setUInt16([vrs["uint16_a"],vrs["uint16_b"]],[1,2])
    fmu.setInt32([vrs["int32_a"],vrs["int32_b"]],[1,2])
    fmu.setUInt32([vrs["uint32_a"],vrs["uint32_b"]],[1,2])
    fmu.setInt64([vrs["int64_a"],vrs["int64_b"]],[1,2])
    fmu.setUInt64([vrs["uint64_a"],vrs["uint64_b"]],[1,2])
    fmu.setBoolean([vrs["boolean_a"],vrs["boolean_b"]],[True,False])
    fmu.setString([vrs["string_a"],vrs["string_b"]],["Hello, ","World!"]) 
    fmu.setBinary([vrs["binary_a"],vrs["binary_b"]],[b"Hello, ",b"World!"])
    
    print(f"Doing a step of size {step_size} at time {sim_time}")
    fmu.doStep(sim_time, step_size)
    sim_time += step_size
    
    print("Fetching output values")
    float32_c = fmu.getFloat32([vrs["float32_c"]])[0]
    float64_c = fmu.getFloat64([vrs["float64_c"]])[0]
    int8_c = fmu.getInt8([vrs["int8_c"]])[0]
    uint8_c = fmu.getUInt8([vrs["uint8_c"]])[0]
    int16_c = fmu.getInt16([vrs["int16_c"]])[0]
    uint16_c = fmu.getUInt16([vrs["uint16_c"]])[0]
    int32_c = fmu.getInt32([vrs["int32_c"]])[0]
    uint32_c = fmu.getUInt32([vrs["uint32_c"]])[0]
    int64_c = fmu.getInt64([vrs["int64_c"]])[0]
    uint64_c = fmu.getUInt64([vrs["uint64_c"]])[0]
    boolean_c = fmu.getBoolean([vrs["boolean_c"]])[0]
    string_c = fmu.getString([vrs["string_c"]])[0]
    
    
    print("Asserting values")
    assert float32_c == 3.0
    assert float64_c == 3.0
    assert int8_c == 3
    assert uint8_c == 3
    assert int16_c == 3
    assert uint16_c == 3
    assert int32_c == 3
    assert uint32_c == 3
    assert int64_c == 3
    assert uint64_c == 3
    assert boolean_c == True
    assert string_c == "Hello, World!"
    
    
    # Restoring initial state
    #print("Restoring initial state")
    #fmu.setFMUState(fmu_state)
    

    print("All tests passed")
    print("Terminating FMU")
    fmu.terminate()
    fmu.freeInstance()
    
    shutil.rmtree(unzipdir)
    
    print("Test completed")
 