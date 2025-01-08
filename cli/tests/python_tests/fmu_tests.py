from fmpy import read_model_description
from fmpy.fmi2 import FMU2Slave
from fmpy.fmi3 import FMU3Slave

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

def fmi2_full_functionality(fmu_filename: str):
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
        print(f"TEST FAILED - fmi2_full_functionality - instantiation: {e}")
        return

    try:

        end_simulation_time = 5.0
        start_simulation_time = 0.0
        threshold = 2.0
        step_size = 0.01

        fmu.setupExperiment(startTime = start_simulation_time)
        fmu.enterInitializationMode()
        fmu.exitInitializationMode()

        simulation_time = start_simulation_time

        vrs = {}
        for variable in model_description.modelVariables:
            vrs[variable.name] = variable.valueReference

        real_c = fmu.getReal([vrs["real_c"]])[0]
        assert real_c == 0.0, f"Before doStep, getReal returned {real_c} which should have been 0.0"
        integer_c = fmu.getInteger([vrs["integer_c"]])[0]
        assert integer_c == 0, f"Before doStep, getInteger returned {real_c} which should have been 0"

        fmu.doStep(
            currentCommunicationPoint = simulation_time,
            communicationStepSize = step_size
        )

        fmu.setReal([vrs["real_a"], vrs["real_b"]], [1.0,2.0])
        fmu.setInteger([vrs["integer_a"], vrs["integer_b"]], [1,2])

        fmu.doStep(
            currentCommunicationPoint = simulation_time,
            communicationStepSize = step_size
        )

        real_c = fmu.getReal([vrs["real_c"]])[0]
        assert real_c == 3.0, f"After doStep, getReal returned {real_c} which should have been 3.0"
        integer_c = fmu.getInteger([vrs["integer_c"]])[0]
        assert integer_c == 3, f"After doStep, getInteger returned {real_c} which should have been 3"
        
    except Exception as e:
        print(f"TEST FAILED - fmi2_full_functionality: {e}")

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
        print(f"TEST FAILED - fmi2_full_functionality - termination: {e}")

if __name__ == "__main__":
    import sys

    function_name = sys.argv[1]
    fmu_filepath = sys.argv[2]

    globals()[function_name](fmu_filepath)