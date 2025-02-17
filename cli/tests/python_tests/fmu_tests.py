from common import barren_test, uninstantiating_test, instantiating_test
from fmpy import extract
from fmpy.model_description import ModelDescription
from fmpy.fmi2 import FMU2Slave
from fmpy.fmi3 import FMU3Slave
from fmpy.fmi2 import fmi2OK

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
def extract_fmu(fmu_filename: str, is_zipped: bool):
    def inner():
        if not is_zipped:
            raise Exception("The given FMU isn't zipped!")
        unzipped_dir = extract(fmu_filename)
    
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
def fmi2_platform(fmu_filename: str, is_zipped: bool):
    def inner(fmu: FMU2Slave):
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
def fmi2_version(fmu_filename: str, is_zipped: bool):
    def inner(fmu: FMU2Slave):
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
def fmi2_instantiate(fmu_filename: str, is_zipped: bool):
    def inner(fmu: FMU2Slave, model_description: ModelDescription):
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
def fmi2_simulate(fmu_filename: str, is_zipped: bool):
    def inner(fmu: FMU2Slave, model_description: ModelDescription):
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
        state = fmu.getFMUstate()
        rerolled_time = sim_time

        print(f"Updating inputs at time {sim_time}")
        fmu.setReal([vrs["real_a"],vrs["real_b"]],[1.0,2.0])
        fmu.setInteger([vrs["integer_a"],vrs["integer_b"]],[1,2])
        fmu.setBoolean([vrs["boolean_a"],vrs["boolean_b"]],[True,False])
        fmu.setString([vrs["string_a"],vrs["string_b"]],["Hello, ","World!"])

        print(f"Doing a step of size {step_size} at time {sim_time}")
        fmu.doStep(sim_time, step_size) == fmi2OK, f"doStep returned with error , should have been return value: {fmi2OK}"
        sim_time += step_size


        print("Fetching evolved values")
        real_c = fmu.getReal([vrs["real_c"]])[0]
        integer_c = fmu.getInteger([vrs["integer_c"]])[0]
        boolean_c = fmu.getBoolean([vrs["boolean_c"]])[0]
        string_c = fmu.getString([vrs["string_c"]])[0].decode("utf-8")

        print("Asserting evolved values")
        assert real_c == 3.0, f"Evolved value real_c was {real_c}, should have been 0.0"
        assert integer_c == 3, f"Evolved value integer_c was {integer_c}, should have been 0"
        assert boolean_c == True, f"Evolved value boolean_c was {boolean_c}, should have been True"
        assert string_c == "Hello, World!", f"Evolved value string_c was '{string_c}', should have been 'Hello, World!'"

        print("Rerolling FMU state")
        fmu.setFMUstate(state)
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

        print("Will doStep and then reset FMU")
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
        # We need to decode strings from bytes to utf-8
        strings = [string.decode("utf-8")  for string in strings]

        print("Asserting values after reset")
        assert reals == [0.0, 0.0, 0.0], f"Initially fetched values were {reals}, should have been [0.0, 0.0, 0.0]"
        assert integers == [0, 0, 0], f"Initially fetched values were {integers}, should have been [0, 0, 0]"
        assert bools == [False, False, False], f"Initially fetched values were {bools}, should have been [False, False, False]"
        assert strings == ["", "", ""], f"Initially fetched values were {strings}, should have been ['', '', '']"

        print(f"Test is done - freeing FMU state")
        fmu.freeFMUState(state)
        print("FMU state freed successfully") 
        print("fmi2_simulate: Test Complete")

        
    instantiating_test(
        caller = "fmi2_simulate",
        inner_function = inner,
        fmu_filename = fmu_filename,
        fmu_class = FMU2Slave,
        is_zipped = is_zipped
    )

"""Asserts that the given FMU is version FMI3.

Parameters
----------
fmu_filename : str
    Full filename of the FMU.
is_zipped : bool
    If true, fmu_filename points to a zip directory. If false, fmu_filename
    points to a normal directory.
"""
def fmi3_version(fmu_filename: str, is_zipped: bool):
    def inner(fmu: FMU3Slave):
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
def fmi3_instantiate(fmu_filename: str, is_zipped: bool):
    def inner(fmu: FMU3Slave, model_description: ModelDescription):
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
def fmi3_simulate(fmu_filename: str, is_zipped: bool):
    def inner(fmu: FMU3Slave, model_description: ModelDescription):
        # can_handle_state = model_description.coSimulation.canGetAndSetFMUstate
        can_handle_state = True

    
        if can_handle_state:
            print("FMU can get and set state")

        start_time = 0.0
        sim_time = start_time
        threshold = 2.0
        step_size = 1e-2

        vrs = {}
        for variable in model_description.modelVariables:
            vrs[variable.name] = variable.valueReference

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
    
        
    instantiating_test(
        caller = "fmi3_simulate",
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

    globals()[function_name](fmu_filepath, is_zipped)