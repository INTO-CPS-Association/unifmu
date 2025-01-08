from fmpy import read_model_description
from fmpy.fmi2 import FMU2Slave
from fmpy.fmi3 import FMU3Slave

def fmi2_platform(fmu_filename: str):
    try:
        model_description = read_model_description(fmu_filename)

        fmu = FMU2Slave(
            guid = model_description.guid,
            unzipDirectory = fmu_filename,
            modelIdentifier = model_description.coSimulation.modelIdentifier,
            instanceName='test_instance'
        )

        platform = fmu.getTypesPlatform()

        assert platform == "default", f"FMU platform was '{platform}', should have been 'default'"

    except Exception as e:
        print(f"TEST FAILED - fmi2_platform: {e}")

def fmi2_version(fmu_filename: str):
    try:
        model_description = read_model_description(fmu_filename)

        fmu = FMU2Slave(
            guid = model_description.guid,
            unzipDirectory = fmu_filename,
            modelIdentifier = model_description.coSimulation.modelIdentifier,
            instanceName='test_instance'
        )

        version = fmu.getVersion()

        assert version == "default", f"FMU version was '{version}', should have been '2.0'"

    except Exception as e:
        print(f"TEST FAILED - fmi2_version: {e}")

def fmi2_instantiate(fmu_filename: str):
    try:
        model_description = read_model_description(fmu_filename)

        fmu = FMU2Slave(
            guid = model_description.guid,
            unzipDirectory = fmu_filename,
            modelIdentifier = model_description.coSimulation.modelIdentifier,
            instanceName='test_instance'
        )

        fmu.instantiate()

    except Exception as e:
        print(f"TEST FAILED - fmi2_instantiate - instantiation: {e}")
        return

    try:
        fmu.terminate()
        fmu.freeInstance()

    except Exception as e:
        print(f"TEST FAILED - fmi2_instantiate - termination: {e}")

def fmi2_simulation(fmu_filename: str):
    try:
        model_description = read_model_description(fmu_filename)

        fmu = FMU2Slave(
            guid = model_description.guid,
            unzipDirectory = fmu_filename,
            modelIdentifier = model_description.coSimulation.modelIdentifier,
            instanceName='test_instance'
        )

        fmu.instantiate()
    
    except Exception as e:
        print(f"TEST FAILED - fmi2_simulation - instantiation: {e}")
        return

    try:
        start_time = 0.0
        sim_time = start_time
        step_size = 1e-2

        can_handle_state = model_description.coSimulation.canGetAndSetFMUstate

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
        assert integer_a == 5, f"Initially fetched value integer_a was {integer_a}, should have been 0"
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
        
    except Exception as e:
        print(f"TEST FAILED - fmi2_simulation: {e}")

        # Blindly try terminating to ensure distributed backend exits.
        # Ignore exceptions as test already failed.
        try:
            fmu.terminate()
            fmu.freeInstance()
        except Exception:
            {}

        return

    try:
        fmu.terminate()
        fmu.freeInstance()

    except Exception as e:
        print(f"TEST FAILED - fmi2_simulation - termination: {e}")

def fmi3_instantiate(fmu_filename: str):
    try:
        model_description = read_model_description(fmu_filename)

        fmu = FMU3Slave(
            guid = model_description.guid,
            unzipDirectory = fmu_filename,
            modelIdentifier = model_description.coSimulation.modelIdentifier,
            instanceName='test_instance'
        )

        fmu.instantiate()

    except Exception as e:
        print(f"TEST FAILED - fmi3_instantiate - instantiation: {e}")
        return

    try:
        fmu.terminate()
        fmu.freeInstance()

    except Exception as e:
        print(f"TEST FAILED - fmi3_instantiate - termination: {e}")

if __name__ == "__main__":
    import sys

    function_name = sys.argv[1]
    fmu_filepath = sys.argv[2]

    globals()[function_name](fmu_filepath)