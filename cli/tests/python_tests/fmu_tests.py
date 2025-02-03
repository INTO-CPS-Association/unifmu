from common import barren_test, uninstantiating_test, instantiating_test
from fmpy import extract
from fmpy.model_description import ModelDescription
from fmpy.fmi2 import FMU2Slave
from fmpy.fmi3 import FMU3Slave

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

        # can_handle_state = model_description.coSimulation.canGetAndSetFMUstate
        can_handle_state = True

        fmu.setupExperiment(startTime=start_time)
        fmu.enterInitializationMode()
        fmu.exitInitializationMode()

        vrs = {}
        for variable in model_description.modelVariables:
            vrs[variable.name] = variable.valueReference

        print("Fetching initial values")
        real_a = fmu.getReal([vrs["real_a"]])[0]
        real_b = fmu.getReal([vrs["real_b"]])[0]
        real_c = fmu.getReal([vrs["real_c"]])[0]

        integer_a = fmu.getInteger([vrs["integer_a"]])[0]
        integer_b = fmu.getInteger([vrs["integer_b"]])[0]
        integer_c = fmu.getInteger([vrs["integer_c"]])[0]

        boolean_a = fmu.getBoolean([vrs["boolean_a"]])[0]
        boolean_b = fmu.getBoolean([vrs["boolean_b"]])[0]
        boolean_c = fmu.getBoolean([vrs["boolean_c"]])[0]

        string_a = fmu.getString([vrs["string_a"]])[0].decode("utf-8")
        string_b = fmu.getString([vrs["string_b"]])[0].decode("utf-8")
        string_c = fmu.getString([vrs["string_c"]])[0].decode("utf-8")

        print("Asserting initial values")
        assert real_a == 0.0, f"Initially fetched value real_a was {real_a}, should have been 0.0"
        assert real_b == 0.0, f"Initially fetched value real_b was {real_b}, should have been 0.0"   
        assert real_c == 0.0, f"Initially fetched value real_c was {real_c}, should have been 0.0"
        assert integer_a == 0, f"Initially fetched value integer_a was {integer_a}, should have been 0"
        assert integer_b == 0, f"Initially fetched value integer_b was {integer_b}, should have been 0"
        assert integer_c == 0, f"Initially fetched value integer_c was {integer_c}, should have been 0"
        assert boolean_a == False, f"Initially fetched value boolean_a was {boolean_a}, should have been False"
        assert boolean_b == False, f"Initially fetched value boolean_b was {boolean_b}, should have been False"
        assert boolean_c == False, f"Initially fetched value boolean_c was {boolean_c}, should have been False"
        assert string_a == "", f"Initially fetched value string_a was '{string_a}', should have been ''"
        assert string_b == "", f"Initially fetched value string_b was '{string_b}', should have been ''"
        assert string_c == "", f"Initially fetched value string_c was '{string_c}', should have been ''"

        if can_handle_state:
            print("Fetching FMU state")
            state = fmu.getFMUstate()
            import logging
            logging.basicConfig(level=logging.DEBUG)
            from ctypes import byref
            logging.debug(f"State is: {byref(state)}")
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

        if can_handle_state: 
            print(f"Reference of state just before rerolling: {byref(state)}")
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
            boolean_c = fmu.getBoolean([vrs["boolean_c"]])[0]

            string_a = fmu.getString([vrs["string_a"]])[0].decode("utf-8")
            string_b = fmu.getString([vrs["string_b"]])[0].decode("utf-8")
            string_c = fmu.getString([vrs["string_c"]])[0].decode("utf-8")

            print("Asserting rerolled values")

            assert real_a == 0.0, f"Rerolled value real_a was {real_a}, should have been 0.0"
            assert real_b == 0.0, f"Rerolled value real_b was {real_b}, should have been 0.0"   
            assert real_c == 0.0, f"Rerolled value real_c was {real_c}, should have been 0.0"
            assert integer_a == 0, f"Rerolled value integer_a was {integer_a}, should have been 0"
            assert integer_b == 0, f"Rerolled value integer_b was {integer_b}, should have been 0"
            assert integer_c == 0, f"Rerolled value integer_c was {integer_c}, should have been 0"
            assert boolean_a == False, f"Rerolled value boolean_a was {boolean_a}, should have been False"
            assert boolean_b == False, f"Rerolled value boolean_b was {boolean_b}, should have been False"
            assert boolean_c == False, f"Rerolled value boolean_c was {boolean_c}, should have been False"
            assert string_a == "", f"Rerolled value string_a was '{string_a}', should have been ''"
            assert string_b == "", f"Rerolled value string_b was '{string_b}', should have been ''"
            assert string_c == "", f"Rerolled value string_c was '{string_c}', should have been ''"
        
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
        can_handle_state = model_description.coSimulation.canGetAndSetFMUstate
        # can_handle_state = True

    
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
        fmu.setBinary(
            [vrs["binary_a"], vrs["binary_b"]],
            [b"Hello, ", b"World!"]
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