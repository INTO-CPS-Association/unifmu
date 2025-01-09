from collections.abc import Callable
from fmpy import read_model_description

def uninstantiating_test(
    caller: str,
    inner_function: Callable,
    fmu_filename: str,
    fmu_class
):
    try:
        model_description = read_model_description(fmu_filename)

        fmu = fmu_class(
            guid = model_description.guid,
            unzipDirectory = fmu_filename,
            modelIdentifier = model_description.coSimulation.modelIdentifier,
            instanceName='test_instance'
        )

        inner_function(fmu)

    except Exception as e:
        print(f"TEST FAILED - {caller}: {e}")


def instantiating_test(
    caller: str, inner_function: Callable, fmu_filename: str, fmu_class
):
    try:
        model_description = read_model_description(fmu_filename)

        fmu = fmu_class(
            guid = model_description.guid,
            unzipDirectory = fmu_filename,
            modelIdentifier = model_description.coSimulation.modelIdentifier,
            instanceName='test_instance'
        )

        fmu.instantiate()

    except Exception as e:
        print(f"TEST FAILED - {caller} - instantiation: {e}")
        return
    
    try:
        inner_function(fmu, model_description)

    except Exception as e:
        print(f"TEST FAILED - {caller}: {e}")

        # Blindly try terminating to ensure distributed backend exits.
        # Ignore exceptions as test already failed.
        try:
            fmu.terminate()
            fmu.freeInstance()
        except Exception:
            pass

        return

    try:
        fmu.terminate()
        fmu.freeInstance()

    except Exception as e:
        print(f"TEST FAILED - {caller} - termination: {e}")
    