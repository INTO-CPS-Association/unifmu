from common import barren_test, uninstantiating_test, instantiating_test
import ctypes
from ctypes import c_uint8, c_ubyte
from fmpy import read_model_description, extract
from fmpy.fmi2 import FMU2Slave
from fmpy.fmi3 import FMU3Slave, fmi3ValueReference, fmi3UInt64, fmi3Float64, fmi3IntervalQualifier
from shutil import rmtree
from fmpy.fmi2 import fmi2OK

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


"""Tries to extract (AKA unzip) the fmu using the fmpy extract function

This will fail if the given FMU is not zipped or if the zipped structure is
wrong.
Can be called with both FMI2 and FMI3 FMUs.

Parameters
----------
fmu_filename : str
    Full filename of the FMU.
is_zipped : bool
    If true, fmu_filename points to a zip directory. If false, fmu_filename
    points to a normal directory.
"""
def extract_fmu(fmu_filename, is_zipped):
    def inner():
        if not is_zipped:
            raise Exception("The given FMU isn't zipped!")
        unzipped_dir = extract(fmu_filename)
        rmtree(unzipped_dir, ignore_errors=True)
    
    barren_test(
        caller = "extract",
        inner_function = inner
    )

"""Asserts that the given FMU was packaged with the correct platform information.

The FMU should conform to FMI2.

Parameters
----------
fmu_filename : str
    Full filename of the FMU.
is_zipped : bool
    If true, fmu_filename points to a zip directory. If false, fmu_filename
    points to a normal directory.
"""
def fmi2_platform(fmu_filename, is_zipped):
    def inner(fmu):
        platform = fmu.getTypesPlatform()

        assert platform == "default", f"FMU platform was '{platform}', should have been 'default'"

    uninstantiating_test(
        caller = "fmi2_platform",
        inner_function = inner,
        fmu_filename = fmu_filename,
        fmu_class = FMU2Slave,
        is_zipped = is_zipped
    )

"""Asserts that the given FMU is version FMI2.

Parameters
----------
fmu_filename : str
    Full filename of the FMU.
is_zipped : bool
    If true, fmu_filename points to a zip directory. If false, fmu_filename
    points to a normal directory.
"""
def fmi2_version(fmu_filename, is_zipped):
    def inner(fmu):
        version = fmu.getVersion()

        assert version == "2.0", f"FMU version was '{version}', should have been '2.0'"
    
    uninstantiating_test(
        caller = "fmi2_version",
        inner_function = inner,
        fmu_filename = fmu_filename,
        fmu_class = FMU2Slave,
        is_zipped = is_zipped
    )

"""Tries to instantiate the FMU.

The FMU should conform to FMI2.

Parameters
----------
fmu_filename : str
    Full filename of the FMU.
is_zipped : bool
    If true, fmu_filename points to a zip directory. If false, fmu_filename
    points to a normal directory.
"""
def fmi2_instantiate(fmu_filename, is_zipped):
    def inner(fmu, model_description):
        pass

    instantiating_test(
        caller = "fmi2_instantiate",
        inner_function = inner,
        fmu_filename = fmu_filename,
        fmu_class = FMU2Slave,
        is_zipped = is_zipped
    )
    
"""Tries to do a full simulation using the FMU.
Will do a simple simulation where the FMU is set up, evolved, rerolled and reset.

The FMU should conform to FMI2.

Parameters
----------
fmu_filename : str
    Full filename of the FMU.
is_zipped : bool
    If true, fmu_filename points to a zip directory. If false, fmu_filename
    points to a normal directory.
"""
def fmi2_simulate(fmu_filename, is_zipped):
    def inner(fmu, model_description):
        start_time = 0.0
        sim_time = start_time
        step_size = 1e-2

        can_handle_state = model_description.coSimulation.canGetAndSetFMUstate
        can_serialize = model_description.coSimulation.canSerializeFMUstate
        
        assert can_handle_state, "FMU cannot get and set state"
        assert can_serialize, "FMU cannot serialize state"

        assert fmu.setupExperiment(startTime=start_time) == fmi2OK, f"setupExperiment returned with error, should have been return value: {fmi2OK}"
        assert fmu.enterInitializationMode() == fmi2OK, f"enterInitializationMode returned with error, should have been return value: {fmi2OK}"
        assert fmu.exitInitializationMode() == fmi2OK, f"exitInitializationMode returned with error, should have been return value: {fmi2OK}"

        vrs = {}
        for variable in model_description.modelVariables:
            vrs[variable.name] = variable.valueReference

        print("Fetching initial values")
        reals = fmu.getReal([vrs["real_a"], vrs["real_b"], vrs["real_c"]])
        integers = fmu.getInteger([vrs["integer_a"], vrs["integer_b"], vrs["integer_c"]])
        bools = fmu.getBoolean([vrs["boolean_a"], vrs["boolean_b"], vrs["boolean_c"]])
        strings = fmu.getString([vrs["string_a"], vrs["string_b"], vrs["string_c"]])
        # We need to decode strings from bytes to utf-8
        strings = [string.decode("utf-8")  for string in strings]

        print("Asserting initial values")
        assert reals == [0.0, 0.0, 0.0], f"Initially fetched values were {reals}, should have been [0.0, 0.0, 0.0]"
        assert integers == [0, 0, 0], f"Initially fetched values were {integers}, should have been [0, 0, 0]"
        assert bools == [False, False, False], f"Initially fetched values were {bools}, should have been [False, False, False]"
        assert strings == ["", "", ""], f"Initially fetched values were {strings}, should have been ['', '', '']"

        print("Fetching FMU state")
        initial_state = fmu.getFMUstate()
        rerolled_time = sim_time

        print(f"Updating inputs at time {sim_time}")
        fmu.setReal([vrs["real_a"],vrs["real_b"]],[1.0,2.0])
        fmu.setInteger([vrs["integer_a"],vrs["integer_b"]],[1,2])
        fmu.setBoolean([vrs["boolean_a"],vrs["boolean_b"]],[True,False])
        fmu.setString([vrs["string_a"],vrs["string_b"]],["Hello, ","World!"])
        updated_state = fmu.getFMUstate() 

        print(f"Doing a step of size {step_size} at time {sim_time}")
        fmu.doStep(sim_time, step_size) == fmi2OK, f"doStep returned with error, should have been return value: {fmi2OK}"
        sim_time += step_size

        print("Fetching evolved values")
        reals = fmu.getReal([vrs["real_a"], vrs["real_b"],vrs["real_c"]])
        integers = fmu.getInteger([vrs["integer_a"], vrs["integer_b"], vrs["integer_c"]])
        booleans = fmu.getBoolean([vrs["boolean_a"], vrs["boolean_b"], vrs["boolean_c"]])
        strings = fmu.getString([vrs["string_a"], vrs["string_b"], vrs["string_c"]])
        strings = [string.decode("utf-8")  for string in strings]

        print("Asserting evolved values")
        assert reals == [1.0, 2.0, 3.0], f"Evolved value real_c was {reals}, should have been [1.0, 2.0, 3.0]"
        assert integers == [1, 2, 3], f"Evolved value integer_c was {integers}, should have been [1, 2, 3]"
        assert booleans == [True, False, True], f"Evolved value boolean_c was {booleans}, should have been [True, False, True]"
        assert strings == ["Hello, ", "World!", "Hello, World!"], f"Evolved value string_c was '{strings}', should have been [Hello, , World!, Hello, World!]"

        print("Rerolling FMU state")
        fmu.setFMUstate(initial_state)
        print("Resetting time")
        sim_time = rerolled_time

        print("Fetching rerolled values")
        reals = fmu.getReal([vrs["real_a"], vrs["real_b"], vrs["real_c"]])
        integers = fmu.getInteger([vrs["integer_a"], vrs["integer_b"], vrs["integer_c"]])
        bools = fmu.getBoolean([vrs["boolean_a"], vrs["boolean_b"], vrs["boolean_c"]])
        strings = fmu.getString([vrs["string_a"], vrs["string_b"], vrs["string_c"]])
        strings = [string.decode("utf-8")  for string in strings]

        print("Asserting rerolled values")
        assert reals == [0.0, 0.0, 0.0], f"Rerolled values were {reals}, should have been [0.0, 0.0, 0.0]"
        assert integers == [0, 0, 0], f"Rerolled values were {integers}, should have been [0, 0, 0]"
        assert bools == [False, False, False], f"Rerolled values were {bools}, should have been [False, False, False]"
        assert strings == ["", "", ""], f"Rerolled values were {strings}, should have been ['', '', '']"

        print("Testing reset")
        fmu.setFMUstate(updated_state)
        fmu.doStep(sim_time, step_size) == fmi2OK, f"doStep returned with error, should have been return value: {fmi2OK}"
        sim_time += step_size
    
        assert fmu.reset() == fmi2OK, f"reset returned with error, should have been return value: {fmi2OK}" 
        # setupExperiment and enterInitializationMode has to be called after reset()
        assert fmu.setupExperiment(startTime=start_time) == fmi2OK, f"setupExperiment returned with error, should have been return value: {fmi2OK}"
        assert fmu.enterInitializationMode() == fmi2OK, f"enterInitializationMode returned with error, should have been return value: {fmi2OK}"
        assert fmu.exitInitializationMode() == fmi2OK, f"exitInitializationMode returned with error, should have been return value: {fmi2OK}"

        print("Fetching values after reset")
        reals = fmu.getReal([vrs["real_a"], vrs["real_b"], vrs["real_c"]])
        integers = fmu.getInteger([vrs["integer_a"], vrs["integer_b"], vrs["integer_c"]])
        bools = fmu.getBoolean([vrs["boolean_a"], vrs["boolean_b"], vrs["boolean_c"]])
        strings = fmu.getString([vrs["string_a"], vrs["string_b"], vrs["string_c"]])
        strings = [string.decode("utf-8")  for string in strings]

        print("Asserting values after reset")
        assert reals == [0.0, 0.0, 0.0], f"Initially fetched values were {reals}, cshould have been [0.0, 0.0, 0.0]"
        assert integers == [0, 0, 0], f"Initially fetched values were {integers}, should have been [0, 0, 0]"
        assert bools == [False, False, False], f"Initially fetched values were {bools}, should have been [False, False, False]"
        assert strings == ["", "", ""], f"Initially fetched values were {strings}, should have been ['', '', '']"

        print(f"Test is done - freeing FMU states")
        fmu.freeFMUState(initial_state)
        fmu.freeFMUState(updated_state)
        print("FMU state freed successfully") 
        print("fmi2_simulate: Test Complete")

        
    instantiating_test(
        caller = "fmi2_simulate",
        inner_function = inner,
        fmu_filename = fmu_filename,
        fmu_class = FMU2Slave,
        is_zipped = is_zipped
    )

"""Tries to instantiate multiple FMUs at once

The FMU should conform to FMI2.

Parameters
----------
fmu_filename : str
    Full filename of the FMU.
is_zipped : bool
    If true, fmu_filename points to a zip directory. If false, fmu_filename
    points to a normal directory.
"""
def fmi2_instantiate_multiple(fmu_filename, is_zipped):
    if is_zipped:
        try:
            fmu_filename = extract(fmu_filename)
        except Exception as e:
            print(f"TEST FAILED - fmi2_instantiate_multiple - zip extraction: {e}")
            return

    try:
        model_description = read_model_description(fmu_filename)

        fmu_1 = FMU2Slave(
            guid = model_description.guid,
            unzipDirectory = fmu_filename,
            modelIdentifier = model_description.coSimulation.modelIdentifier,
            instanceName='test_instance'
        )

        fmu_2 = FMU2Slave(
            guid = model_description.guid,
            unzipDirectory = fmu_filename,
            modelIdentifier = model_description.coSimulation.modelIdentifier,
            instanceName='test_instance'
        )

        fmu_3 = FMU2Slave(
            guid = model_description.guid,
            unzipDirectory = fmu_filename,
            modelIdentifier = model_description.coSimulation.modelIdentifier,
            instanceName='test_instance'
        )

        fmu_1.instantiate(loggingOn=True)
        fmu_2.instantiate(loggingOn=True)
        fmu_3.instantiate(loggingOn=True)

    except Exception as e:
        print(f"TEST FAILED - fmi2_instantiate_multiple - instantiation: {e}")

        if is_zipped:
            rmtree(fmu_filename, ignore_errors=True)

        return

    try:
        fmu_1.terminate()
        fmu_1.freeInstance()
        fmu_2.terminate()
        fmu_2.freeInstance()
        fmu_3.terminate()
        fmu_3.freeInstance()

    except Exception as e:
        print(f"TEST FAILED - fmi2_instantiate_multiple - termination: {e}")

    if is_zipped:
        rmtree(fmu_filename, ignore_errors=True)

"""Asserts that the given FMU is version FMI3.

Parameters
----------
fmu_filename : str
    Full filename of the FMU.
is_zipped : bool
    If true, fmu_filename points to a zip directory. If false, fmu_filename
    points to a normal directory.
"""
def fmi3_version(fmu_filename, is_zipped):
    def inner(fmu):
        version = fmu.getVersion()

        assert version == "3.0", f"FMU version was '{version}', should have been '3.0'"
    
    uninstantiating_test(
        caller = "fmi3_version",
        inner_function = inner,
        fmu_filename = fmu_filename,
        fmu_class = FMU3Slave,
        is_zipped = is_zipped
    )

"""Tries to instantiate the FMU.

The FMU should conform to FMI3.

Parameters
----------
fmu_filename : str
    Full filename of the FMU.
is_zipped : bool
    If true, fmu_filename points to a zip directory. If false, fmu_filename
    points to a normal directory.
"""
def fmi3_instantiate(fmu_filename, is_zipped):
    def inner(fmu, model_description):
        pass

    instantiating_test(
        caller = "fmi3_instantiate",
        inner_function = inner,
        fmu_filename = fmu_filename,
        fmu_class = FMU3Slave,
        is_zipped = is_zipped
    )

"""Tries to do a full simulation using the FMU.

The FMU should conform to FMI3.

Parameters
----------
fmu_filename : str
    Full filename of the FMU.
is_zipped : bool
    If true, fmu_filename points to a zip directory. If false, fmu_filename
    points to a normal directory.
"""
def fmi3_simulate(fmu_filename, is_zipped):
    def inner(fmu, model_description):
        can_handle_state = model_description.coSimulation.canGetAndSetFMUstate
    
        #if can_handle_state:
            #print("FMU can get and set state")

        start_time = 0.0
        sim_time = start_time
        step_size = 1e-2

        vrs = {}
        for variable in model_description.modelVariables:
            vrs[variable.name] = variable.valueReference

        fmu.enterInitializationMode()
        fmu.exitInitializationMode()

        # Fetching initial values from the FMU
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

        matrix_a = fmu.getFloat32([vrs["matrix_a"]], 9)
        matrix_b = fmu.getFloat32([vrs["matrix_b"]], 24)

        # Asserting initial values
        assert float32 == [0.0, 0.0, 0.0], f"Initially fetched float32s were {float32}, should have been [0.0, 0.0, 0.0]."
        assert float64 == [0.0, 0.0, 0.0], f"Initially fetched float64s were {float64}, should have been [0.0, 0.0, 0.0]."
    
        assert int8 == [0, 0, 0], f"Initially fetched int8s were {int8}, should have been [0, 0, 0]."
        assert uint8 == [0, 0, 0], f"Initially fetched uint8s were {uint8}, should have been [0, 0, 0]."
        assert int16 == [0, 0, 0], f"Initially fetched int16s were {int16}, should have been [0, 0, 0]."
        assert uint16 == [0, 0, 0], f"Initially fetched uint16s were {uint16}, should have been [0, 0, 0]."
        assert int32 == [0, 0, 0], f"Initially fetched int32s were {int32}, should have been [0, 0, 0]."
        assert uint32 == [0, 0, 0], f"Initially fetched uint32s were {uint32}, should have been [0, 0, 0]."
        assert int64 == [0, 0, 0], f"Initially fetched int64s were {int64}, should have been [0, 0, 0]."
        assert uint64 == [0, 0, 0], f"Initially fetched uint64s were {uint64}, should have been [0, 0, 0]."

        assert matrix_a == [1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0, 34.0, 55.0], f"Initially fetched matrix_a was {matrix_a}, should have been [1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0, 34.0, 55.0]."
        assert matrix_b == [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0], f"Initially fetched matrix_b was {matrix_b}, should have been [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0]."

        assert boolean == [False, False, False], f"Initially fetched booleans were {boolean}, should have been [False, False, False]."

        assert string == ["", "", ""], f"Initially fetched strings were {string}, should have been [\"\", \"\", \"\"]."

        binary_should_be = [bytes(c_uint8(0)), bytes(c_uint8(0)), bytes(c_uint8(0))]
        assert binary == binary_should_be, f"Initially fetched binaries were {binary}, should have been {binary_should_be}."

        # Simulating

        # Updating inputs
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
            [(c_ubyte * 4)(10, 20, 30, 40), (c_ubyte * 4)(15, 25, 35, 45)]
        )
        fmu.setFloat32(
            [vrs["matrix_a"]],
            [8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0]
        )
    
        # Doing a step
        fmu.doStep(sim_time, step_size)
        sim_time += step_size
    
        # Fetching output values
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
        
        # Asserting output values
        assert float32_c == 3.0, f"fetched float32_c was {float32_c}, should have been 3.0."
        assert float64_c == 3.0, f"fetched float64_c was {float64_c}, should have been 3.0."
        assert int8_c == 3, f"fetched int8_c was {int8_c}, should have been 3."
        assert uint8_c == 3, f"fetched uint8_c was {uint8_c}, should have been 3."
        assert int16_c == 3, f"fetched int16_c was {int16_c}, should have been 3."
        assert uint16_c == 3, f"fetched uint16_c was {uint16_c}, should have been 3."
        assert int32_c == 3, f"fetched int32_c was {int32_c}, should have been 3."
        assert uint32_c == 3, f"fetched uint32_c was {uint32_c}, should have been 3."
        assert int64_c == 3, f"fetched int64_c was {int64_c}, should have been 3."
        assert uint64_c == 3, f"fetched uint64_c was {uint64_c}, should have been 3."
        assert boolean_c == True, f"fetched boolean_c was {boolean_c}, should have been True."
        assert string_c == "Hello, World!", f"fetched string_c was {string_c}, should have been \"Hello, World!\"."
        binary_should_be = bytes((c_ubyte * 4)(5, 13, 61, 5))
        assert binary_c == binary_should_be, f"fetched binary_c was {binary_c.hex()}, should have been {binary_should_be.hex()}."
        
        binary = fmu.getBinary([
            vrs["binary_a"], vrs["binary_b"], vrs["binary_c"]
        ])
        
        binary_should_be = [bytes((c_ubyte * 4)(10, 20, 30, 40)), bytes((c_ubyte * 4)(15, 25, 35, 45)), bytes((c_ubyte * 4)(5, 13, 61, 5))]
        assert binary == binary_should_be, f"fetched binaries was {binary}, should have been {binary_should_be}."

         ## Testing state-related functions ##
        can_handle_state = model_description.coSimulation.canGetAndSetFMUstate
        can_serialize = model_description.coSimulation.canSerializeFMUstate

        assert can_handle_state, "FMU cannot get and set state"
        assert can_serialize, "FMU cannot serialize state"
        saved_state = fmu.getFMUState()
        #print("saved state: " + str(saved_state))
        reroll_time = sim_time

        # Updating inputs
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
        fmu.setFloat32(
            [vrs["matrix_a"]],
            [9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0]
        )

        # Doing a step
        fmu.doStep(sim_time, step_size)
        sim_time += step_size

        # Fetching output values
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

        # Asserting output values (before setting the state)
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

        # Fetching and asserting matrix value
        matrix_a = fmu.getFloat32([vrs["matrix_a"]], 9)
        assert matrix_a == [9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0], f"Matrix_a after roundtrip was {matrix_a}, should have been [9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0]."

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
        fmu.setFMUState(saved_state)
        sim_time = reroll_time

        # Fetching output values (after rollback)
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

        # Asserting output values (after rollback)
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

        # Fetching and asserting matrix value (after rollback)
        matrix_a = fmu.getFloat32([vrs["matrix_a"]], 9)
        assert matrix_a == [8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0], f"Matrix was {matrix_a} after rollback, should have been [8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0]."

        # Resetting state
        reset_return = fmu.reset()
        enter_init_return = fmu.enterInitializationMode()
        exit_init_return = fmu.exitInitializationMode()

        # Fetching values after resetting
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

        # Asserting initial values after resetting
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

        # Fetching and asserting matrix value (after resetting)
        matrix_a = fmu.getFloat32([vrs["matrix_a"]], 9)
        assert matrix_a == [1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0, 34.0, 55.0], f"Matrix was {matrix_a} after reset, should have been [1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0, 34.0, 55.0]."

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

        # Tests for event mode
        fmu.getInt32([vrs["clocked_variable_c"]]) # Should warn (not in event mode) 

        fmu.enterEventMode()
        clocked_variable_c = fmu.getInt32([vrs["clocked_variable_c"]])[0]
        assert clocked_variable_c == 0

        fmu.setInt32([vrs["clocked_variable_a"],vrs["clocked_variable_b"]],[1,2])
        fmu.updateDiscreteStates()
        clocked_variable_c = fmu.getInt32([vrs["clocked_variable_c"]])[0]
        assert clocked_variable_c == 3

        # Check the update of tunable parameters
        boolean_tunable_parameter = fmu.getBoolean([vrs["boolean_tunable_parameter"]])[0]
        assert boolean_tunable_parameter == False

        fmu.setBoolean([vrs["boolean_tunable_parameter"]],[True])
        boolean_tunable_parameter = fmu.getBoolean([vrs["boolean_tunable_parameter"]])[0]
        assert boolean_tunable_parameter == True

        fmu.enterStepMode()

    instantiating_test(
        caller = "fmi3_simulate",
        inner_function = inner,
        fmu_filename = fmu_filename,
        fmu_class = FMU3Slave,
        is_zipped = is_zipped
    )

"""Tries to get and set a 2-dimensional matrix value for the FMU.

The FMU should conform to FMI3.

Parameters
----------
fmu_filename : str
    Full filename of the FMU.
is_zipped : bool
    If true, fmu_filename points to a zip directory. If false, fmu_filename
    points to a normal directory.
"""
def fmi3_matrix_operations(fmu_filename, is_zipped):
    def inner(fmu, model_description):
        vrs = {}
        for variable in model_description.modelVariables:
            vrs[variable.name] = variable.valueReference

        fmu.enterInitializationMode()
        fmu.exitInitializationMode()

        # Testing initial values
        matrix_a = fmu.getFloat32([vrs["matrix_a"]], 9)
        matrix_b = fmu.getFloat32([vrs["matrix_b"]], 24)

        assert matrix_a == [1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0, 34.0, 55.0], f"Initially fetched matrix_a was {matrix_a}, should have been [1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0, 34.0, 55.0]."
        assert matrix_b == [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0], f"Initially fetched matrix_b was {matrix_b}, should have been [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0]."

        # Testing functionality of example matrix in model
        sim_time = 0.0
        step_size = 1e-2
        fmu.doStep(sim_time, step_size)
        sim_time += step_size
        matrix_c = fmu.getFloat32([vrs["matrix_c"]], 3)
        assert matrix_c[0] == 215, f"First value of matrix_c after dostep was {matrix_c[0]}, should have been 215. (matrix_c: {matrix_c})"
        fmu.doStep(sim_time, step_size)
        sim_time += step_size
        matrix_c = fmu.getFloat32([vrs["matrix_c"]], 3)
        assert matrix_c[0] == 1295, f"First value of matrix_c after dostep was {matrix_c[0]}, should have been 1295. (matrix_c: {matrix_c})"

        # Testing roundtrip
        fmu.setFloat32(
            [vrs["matrix_a"]],
            [9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0]
        )
        fmu.setFloat32(
            [vrs["matrix_b"]],
            [2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0, 26.0, 28.0, 30.0, 32.0, 34.0, 36.0, 38.0, 40.0, 42.0, 44.0, 46.0, 48.0]
        )

        matrix_a = fmu.getFloat32([vrs["matrix_a"]], 9)
        matrix_b = fmu.getFloat32([vrs["matrix_b"]], 24)

        assert matrix_a == [9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0], f"Matrix_a after roundtrip was {matrix_a}, should have been [9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0]."
        assert matrix_b == [2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0, 26.0, 28.0, 30.0, 32.0, 34.0, 36.0, 38.0, 40.0, 42.0, 44.0, 46.0, 48.0], f"Matrix_b after roundtrip was {matrix_b}, should have been [2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0, 26.0, 28.0, 30.0, 32.0, 34.0, 36.0, 38.0, 40.0, 42.0, 44.0, 46.0, 48.0]."

    instantiating_test(
        caller = "fmi3_matrix_operations",
        inner_function = inner,
        fmu_filename = fmu_filename,
        fmu_class = FMU3Slave,
        is_zipped = is_zipped
    )

if __name__ == "__main__":
    import sys

    function_name = sys.argv[1]
    fmu_filepath = sys.argv[2]
    is_zipped = sys.argv[3]
    if is_zipped == "true" or is_zipped == "True":
        is_zipped = True
    else:
        is_zipped = False

    # When these tests are called from the rust test module, python runs them
    # noninteractively (as there are no parts of the code that ask for
    # interaction from any human). When python code is run noninteractively all
    # messages send to stdout are buffered and held until execution ends.
    # This interferes with our tests - distributed UniFMUs have an interactive
    # step where the "local" part sends a message to stdout that is needed to
    # start up the "remote" part. We automate this interaction (see the rust
    # common module for tests), and as such we need the messages from python
    # to be emitted when printed instead of at end of execution.
    # This line ensures that prints are emitted at line end, which 
    # is sufficient.
    sys.stdout.reconfigure(line_buffering=True)

    globals()[function_name](fmu_filepath, is_zipped)