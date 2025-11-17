from fmpy import read_model_description, extract
from fmpy.fmi3 import FMU3Slave,fmi3OK, fmi3ValueReference, fmi3Binary, fmi3Error, fmi3UInt64,fmi3Float64,fmi3IntervalQualifier, fmi3Instance
import shutil
import sys
import logging
import ctypes
from ctypes import c_uint8

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__file__)

## Overwrites the clock related functions
def getIntervalDecimal(fmu,valueReferences):
        nValueReferences = len(valueReferences)
        valueReferences = (fmi3ValueReference * nValueReferences)(*valueReferences)
        intervals = (fmi3Float64 * nValueReferences)()
        qualifiers = (fmi3IntervalQualifier * nValueReferences)()
        fmu.fmi3GetIntervalDecimal(fmu.component, valueReferences, nValueReferences, intervals, qualifiers)
        return list(intervals),list(qualifiers)

def getIntervalFraction(fmu,valueReferences):
    nValueReferences = len(valueReferences)
    valueReferences = (fmi3ValueReference * nValueReferences)(*valueReferences)
    counters = (fmi3UInt64 * nValueReferences)()
    resolutions = (fmi3UInt64 * nValueReferences)()
    qualifiers = (fmi3IntervalQualifier * nValueReferences)()
    fmu.fmi3GetIntervalFraction(fmu.component, valueReferences, nValueReferences, counters, resolutions, qualifiers)
    return list(counters),list(resolutions),list(qualifiers)

def getShiftDecimal(fmu,valueReferences):
    nValueReferences = len(valueReferences)
    valueReferences = (fmi3ValueReference * nValueReferences)(*valueReferences)
    shifts = (fmi3Float64 * nValueReferences)()
    fmu.fmi3GetShiftDecimal(fmu.component, valueReferences, nValueReferences, shifts)
    return list(shifts)

def getShiftFraction(fmu,valueReferences):
    nValueReferences = len(valueReferences)
    valueReferences = (fmi3ValueReference * nValueReferences)(*valueReferences)
    counters = (fmi3UInt64 * nValueReferences)()
    resolutions = (fmi3UInt64 * nValueReferences)()
    fmu.fmi3GetShiftFraction(fmu.component, valueReferences, nValueReferences, counters, resolutions)
    return list(counters),list(resolutions)

def setIntervalDecimal(fmu,valueReferences, intervals):
    nValueReferences = len(valueReferences)
    valueReferences = (fmi3ValueReference * nValueReferences)(*valueReferences)
    intervals = (fmi3Float64 * nValueReferences)(*intervals)
    fmu.fmi3SetIntervalDecimal(fmu.component, valueReferences, nValueReferences, intervals)

def setIntervalFraction(fmu,valueReferences, counters, resolutions):
    nValueReferences = len(valueReferences)
    valueReferences = (fmi3ValueReference * nValueReferences)(*valueReferences)
    counters = (fmi3UInt64 * nValueReferences)(*counters)
    resolutions = (fmi3UInt64 * nValueReferences)(*resolutions)
    fmu.fmi3SetIntervalFraction(fmu.component, valueReferences, nValueReferences, counters, resolutions)

def setShiftDecimal(fmu,valueReferences, shifts):
    nValueReferences = len(valueReferences)
    valueReferences = (fmi3ValueReference * nValueReferences)(*valueReferences)
    shifts = (fmi3Float64 * nValueReferences)(*shifts)
    fmu.fmi3SetShiftDecimal(fmu.component, valueReferences, nValueReferences, shifts)

def setShiftFraction(fmu,valueReferences, counters, resolutions):
    nValueReferences = len(valueReferences)
    valueReferences = (fmi3ValueReference * nValueReferences)(*valueReferences)
    counters = (fmi3UInt64 * nValueReferences)(*counters)
    resolutions = (fmi3UInt64 * nValueReferences)(*resolutions)
    fmu.fmi3SetShiftFraction(fmu.component, valueReferences, nValueReferences, counters, resolutions)

if __name__ == "__main__":
    input_ok = False
    if len(sys.argv)==2:
        fmu_filename = str(sys.argv[1])
    end_simulation_time = 5.0
    start_simulation_time = 0.0
    sim_time = start_simulation_time
    step_size = 0.01
    # extract the FMU
    unzipdir = extract(fmu_filename)

    # read the model description
    model_description = read_model_description(unzipdir)

    # collect the value references
    vrs = {}
    for variable in model_description.modelVariables:
        vrs[variable.name] = variable.valueReference

   

    fmu = FMU3Slave(guid=model_description.guid,
                    unzipDirectory=unzipdir,
                    modelIdentifier=model_description.coSimulation.modelIdentifier,
                    instanceName='instance1')

    # initialize
    fmu.instantiate(visible=False,
                    loggingOn=True,
                    eventModeUsed=True,
                    earlyReturnAllowed=True,
                    logMessage=None,
                    intermediateUpdate=None)
    fmu.enterInitializationMode()
    fmu.exitInitializationMode()

    print("Fetching initial values from the FMU")
    
    float32 = fmu.getFloat32([
        vrs["float32_a"], vrs["float32_b"], vrs["float32_c"]
    ])
    float64 = fmu.getFloat64([
        vrs["float64_a"], vrs["float64_b"], vrs["float64_a"]
    ])

    int8 = fmu.getInt8([
        vrs["int8_a"], vrs["int8_b"], vrs["int8_c"]
    ])
    uint8 = fmu.getUInt8([
        vrs["uint8_a"], vrs["uint8_b"], vrs["uint8_c"]
    ])
    int16 = fmu.getInt16([
        vrs["int16_a"], vrs["int16_b"], vrs["int16_c"]
    ])
    uint16 = fmu.getUInt16([
        vrs["uint16_a"], vrs["uint16_b"], vrs["uint16_c"]
    ])
    int32 = fmu.getInt32([
        vrs["int32_a"], vrs["int32_b"], vrs["int32_c"]
    ])
    uint32 = fmu.getUInt32([
        vrs["uint32_a"], vrs["uint32_b"], vrs["uint32_c"]
    ])
    int64 = fmu.getInt64([
        vrs["int64_a"], vrs["int64_b"], vrs["int64_c"]
    ])
    uint64 = fmu.getUInt64([
        vrs["uint64_a"], vrs["uint64_b"], vrs["uint64_c"]
    ])

    boolean = fmu.getBoolean([
        vrs["boolean_a"], vrs["boolean_b"], vrs["boolean_c"]
    ])
    string = fmu.getString([
        vrs["string_a"], vrs["string_b"], vrs["string_c"]
    ])

    binary = fmu.getBinary([
        vrs["binary_a"], vrs["binary_b"], vrs["binary_c"]
    ])

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
    assert binary == [bytes(c_uint8(0)), bytes(c_uint8(0)), bytes(c_uint8(0))]

    # Simulating
    print(f"Updating inputs at time {sim_time}")

    fmu.setFloat32(
        [vrs["float32_a"], vrs["float32_b"]],
        [1.0, 2.0]
    )
    fmu.setFloat64(
        [vrs["float64_a"], vrs["float64_b"]],
        [1.0, 2.0]
    )
    fmu.setInt8(
        [vrs["int8_a"], vrs["int8_b"]],
        [1, 2]
    )
    fmu.setUInt8(
        [vrs["uint8_a"], vrs["uint8_b"]],
        [1, 2]
    )
    fmu.setInt16(
        [vrs["int16_a"], vrs["int16_b"]],
        [1, 2]
    )
    fmu.setUInt16(
        [vrs["uint16_a"], vrs["uint16_b"]],
        [1, 2]
    )
    fmu.setInt32(
        [vrs["int32_a"], vrs["int32_b"]],
        [1, 2]
    )
    fmu.setUInt32(
        [vrs["uint32_a"], vrs["uint32_b"]],
        [1, 2]
    )
    fmu.setInt64(
        [vrs["int64_a"], vrs["int64_b"]],
        [1, 2]
    )
    fmu.setUInt64(
        [vrs["uint64_a"], vrs["uint64_b"]],
        [1, 2]
    )
    fmu.setBoolean(
        [vrs["boolean_a"], vrs["boolean_b"]],
        [True, False]
    )
    fmu.setString(
        [vrs["string_a"], vrs["string_b"]],
        ["Hello, ", "World!"]
    ) 
    fmu.setBinary(
        [vrs["binary_a"], vrs["binary_b"]],
        [(ctypes.c_ubyte * 4)(10, 20, 30, 40), (ctypes.c_ubyte * 4)(15, 25, 35, 45)]
    ) 
    

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
    binary_c = fmu.getBinary([vrs["binary_c"]])[0]

    print("Asserting output values")
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
    assert binary_c == bytes((ctypes.c_ubyte * 4)(5, 13, 61, 5))

    binary = fmu.getBinary([
        vrs["binary_a"], vrs["binary_b"], vrs["binary_c"]
    ])

    assert binary == [bytes((ctypes.c_ubyte * 4)(10, 20, 30, 40)), bytes((ctypes.c_ubyte * 4)(15, 25, 35, 45)), bytes((ctypes.c_ubyte * 4)(5, 13, 61, 5))]
    
    ## Testing state-related functions ##
    can_handle_state = model_description.coSimulation.canGetAndSetFMUstate
    can_serialize = model_description.coSimulation.canSerializeFMUstate
    
    assert can_handle_state, "FMU cannot get and set state"
    assert can_serialize, "FMU cannot serialize state"
    saved_state = fmu.getFMUState()
    print("saved state: " + str(saved_state))
    reroll_time = sim_time
    print(f"Updating inputs at time {sim_time}")

    fmu.setFloat32(
        [vrs["float32_a"], vrs["float32_b"]],
        [2.0, 3.0]
    )
    fmu.setFloat64(
        [vrs["float64_a"], vrs["float64_b"]],
        [2.0, 3.0]
    )
    fmu.setInt8(
        [vrs["int8_a"], vrs["int8_b"]],
        [2, 3]
    )
    fmu.setUInt8(
        [vrs["uint8_a"], vrs["uint8_b"]],
        [2, 3]
    )
    fmu.setInt16(
        [vrs["int16_a"], vrs["int16_b"]],
        [2, 3]
    )
    fmu.setUInt16(
        [vrs["uint16_a"], vrs["uint16_b"]],
        [2, 3]
    )
    fmu.setInt32(
        [vrs["int32_a"], vrs["int32_b"]],
        [2, 3]
    )
    fmu.setUInt32(
        [vrs["uint32_a"], vrs["uint32_b"]],
        [2, 3]
    )
    fmu.setInt64(
        [vrs["int64_a"], vrs["int64_b"]],
        [2, 3]
    )
    fmu.setUInt64(
        [vrs["uint64_a"], vrs["uint64_b"]],
        [2, 3]
    )
    fmu.setBoolean(
        [vrs["boolean_a"], vrs["boolean_b"]],
        [True, False]
    )
    fmu.setString(
        [vrs["string_a"], vrs["string_b"]],
        ["Hello, ", "World!"]
    ) 
    fmu.setBinary(
        [vrs["binary_a"], vrs["binary_b"]],
        [(ctypes.c_ubyte * 1)(15), (ctypes.c_ubyte * 1)(16)]
    ) 

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
    binary_c = fmu.getBinary([vrs["binary_c"]])[0]

    print("Asserting output values (before setting the state)")
    assert float32_c == 5.0
    assert float64_c == 5.0
    assert int8_c == 5
    assert uint8_c == 5
    assert int16_c == 5
    assert uint16_c == 5
    assert int32_c == 5
    assert uint32_c == 5
    assert int64_c == 5
    assert uint64_c == 5
    assert boolean_c == True
    assert string_c == "Hello, World!"
    assert binary_c == bytes(ctypes.c_ubyte(31))

    # Entering in configuration mode
    fmu.fmi3EnterConfigurationMode(fmu.component)
    uint64_tunable_structural_parameter = fmu.getUInt64([vrs["uint64_tunable_structural_parameter"]])[0]
    assert uint64_tunable_structural_parameter == 5

    fmu.setUInt64([vrs["uint64_tunable_structural_parameter"]],[6])
    uint64_tunable_structural_parameter = fmu.getUInt64([vrs["uint64_tunable_structural_parameter"]])[0]
    assert uint64_tunable_structural_parameter == 6
    # Exiting configuration mode
    fmu.fmi3ExitConfigurationMode(fmu.component)

    fmu.setUInt64([vrs["uint64_tunable_structural_parameter"]],[7]) # Should warn (not in config mode)

    # Setting state to previous state
    print("Setting to a previous state")
    fmu.setFMUState(saved_state)
    sim_time = reroll_time

    print("Fetching output values (after rollback)")
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
    binary_c = fmu.getBinary([vrs["binary_c"]])[0]

    print("Asserting output values (after rollback)")
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
    print(f'binary_c:  {list(binary_c)}')
    assert binary_c == bytes((ctypes.c_ubyte * 4)(5, 13, 61, 5))

    print("Resetting state")
    reset_return = fmu.reset()
    enter_init_return = fmu.enterInitializationMode()
    exit_init_return = fmu.exitInitializationMode()
    print("Fetching values after resetting")
    float32 = fmu.getFloat32([
        vrs["float32_a"], vrs["float32_b"], vrs["float32_c"]
    ])
    float64 = fmu.getFloat64([
        vrs["float64_a"], vrs["float64_b"], vrs["float64_a"]
    ])

    int8 = fmu.getInt8([
        vrs["int8_a"], vrs["int8_b"], vrs["int8_c"]
    ])
    uint8 = fmu.getUInt8([
        vrs["uint8_a"], vrs["uint8_b"], vrs["uint8_c"]
    ])
    int16 = fmu.getInt16([
        vrs["int16_a"], vrs["int16_b"], vrs["int16_c"]
    ])
    uint16 = fmu.getUInt16([
        vrs["uint16_a"], vrs["uint16_b"], vrs["uint16_c"]
    ])
    int32 = fmu.getInt32([
        vrs["int32_a"], vrs["int32_b"], vrs["int32_c"]
    ])
    uint32 = fmu.getUInt32([
        vrs["uint32_a"], vrs["uint32_b"], vrs["uint32_c"]
    ])
    int64 = fmu.getInt64([
        vrs["int64_a"], vrs["int64_b"], vrs["int64_c"]
    ])
    uint64 = fmu.getUInt64([
        vrs["uint64_a"], vrs["uint64_b"], vrs["uint64_c"]
    ])

    boolean = fmu.getBoolean([
        vrs["boolean_a"], vrs["boolean_b"], vrs["boolean_c"]
    ])
    string = fmu.getString([
        vrs["string_a"], vrs["string_b"], vrs["string_c"]
    ])

    binary = fmu.getBinary([
        vrs["binary_a"], vrs["binary_b"], vrs["binary_c"]
    ])

    print("Asserting initial values after resetting")
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
    assert binary == [bytes(c_uint8(0)), bytes(c_uint8(0)), bytes(c_uint8(0))]

    print("Tests for clock-related functions")
    interval_decimals = getIntervalDecimal(fmu,[vrs["clock_a"]])
    interval_fractions = getIntervalFraction(fmu,[vrs["clock_a"]])
    shift_decimals = getShiftDecimal(fmu,[vrs["clock_a"]])
    shift_fractions = getShiftFraction(fmu,[vrs["clock_a"]])

    assert interval_decimals == ([1.0],[2])
    assert interval_fractions == ([1],[1],[2])
    assert shift_decimals == [1.0]
    assert shift_fractions == ([1],[1])

    setIntervalDecimal(fmu,[vrs["clock_a"]],[1.5])
    setShiftDecimal(fmu,[vrs["clock_a"]],[1.5])

    interval_decimals = getIntervalDecimal(fmu,[vrs["clock_a"]])
    interval_fractions = getIntervalFraction(fmu,[vrs["clock_a"]])
    shift_decimals = getShiftDecimal(fmu,[vrs["clock_a"]])
    shift_fractions = getShiftFraction(fmu,[vrs["clock_a"]])
    assert interval_decimals == ([1.5],[2])
    assert interval_fractions == ([3],[2],[2])
    assert shift_decimals == [1.5]
    assert shift_fractions == ([3],[2])

    setIntervalFraction(fmu,[vrs["clock_a"]],[5],[2])
    setShiftFraction(fmu,[vrs["clock_a"]],[5],[2])

    interval_decimals = getIntervalDecimal(fmu,[vrs["clock_a"]])
    interval_fractions = getIntervalFraction(fmu,[vrs["clock_a"]])
    shift_decimals = getShiftDecimal(fmu,[vrs["clock_a"]])
    shift_fractions = getShiftFraction(fmu,[vrs["clock_a"]])
    assert interval_decimals == ([2.5],[2])
    assert interval_fractions == ([5],[2],[2])
    assert shift_decimals == [2.5]
    assert shift_fractions == ([5],[2])

    print("Tests for event mode")
    fmu.getUInt32([vrs["clocked_variable_c"]]) # Should warn (not in event mode) 

    fmu.enterEventMode()
    clocked_variable_c = fmu.getInt32([vrs["clocked_variable_c"]])[0]
    print(f'clocked_variable_c: {clocked_variable_c}')
    assert clocked_variable_c == 0

    fmu.setInt32([vrs["clocked_variable_a"],vrs["clocked_variable_b"]],[1,2])
    fmu.updateDiscreteStates()
    clocked_variable_c = fmu.getInt32([vrs["clocked_variable_c"]])[0]
    print(f'clocked_variable_c: {clocked_variable_c}')
    assert clocked_variable_c == 3

    # Check the update of tunable parameters
    boolean_tunable_parameter = fmu.getBoolean([vrs["boolean_tunable_parameter"]])[0]
    assert boolean_tunable_parameter == False

    fmu.setBoolean([vrs["boolean_tunable_parameter"]],[True])
    boolean_tunable_parameter = fmu.getBoolean([vrs["boolean_tunable_parameter"]])[0]
    assert boolean_tunable_parameter == True

    fmu.enterStepMode()

    
    # terminate
    fmu.terminate()
    fmu.freeInstance()
    # clean up
    shutil.rmtree(unzipdir, ignore_errors=True)
