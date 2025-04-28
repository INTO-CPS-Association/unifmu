from fmpy import read_model_description, extract
from shutil import rmtree

"""Test wrapper that does nothing except transforming exceptions into error
messages understood by the rust test framework.

Parameters
----------
caller : str
    Name of the outer function. Used in failure messages to specify where the
    failure was.
inner_function : Callable
    Function containing the actual tests. Any exception in this function is
    treated as test failure.
"""
def barren_test(caller, inner_function):
    try:
        inner_function()
    except Exception as e:
        print(f"TEST FAILED - {caller}: {e}")

"""Test wrapper that only imports the FMU without instantiating it.

Parameters
----------
caller : str
    Name of the outer function. Used in failure messages to specify where the
    failure was.
inner_function : Callable[[FMU2Slave | FMU3Slave]]
    Function containing the actual tests. Any exception in this function is
    treated as test failure.
fmu_filename : str
    Full filename of the file containing the FMU. Currently the tests assume an
    unzipped FMU, so this should be the full name of the unzipped FMU directory.
fmu_class : FMU2Slave | FMU3Slave
    Class name of the fmpy FMU object to create from the given fmu_filename.
"""
def uninstantiating_test(
    caller,
    inner_function,
    fmu_filename,
    fmu_class,
    is_zipped = False
):
    if is_zipped:
        try:
            fmu_filename = extract(fmu_filename)
        except Exception as e:
            print(f"TEST FAILED - {caller} - zip extraction: {e}")
            return

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

    if is_zipped:
        rmtree(fmu_filename, ignore_errors=True)

"""Test wrapper that imports and instantiates the FMU.

Parameters
----------
caller : str
    Name of the outer function. Used in failure messages to specify where the
    failure was.
inner_function :  Callable[[FMU2Slave | FMU3Slave, ModelDescription]]
    Function containing the actual tests. Any exception in this function is
    treated as test failure.
fmu_filename : str
    Full filename of the file containing the FMU. Currently the tests assume an
    unzipped FMU, so this should be the full name of the unzipped FMU directory.
fmu_class : FMU2Slave | FMU3Slave
    Class name of the fmpy FMU object to create from the given fmu_filename.
"""
def instantiating_test(
    caller,
    inner_function,
    fmu_filename,
    fmu_class,
    is_zipped = False
):
    if is_zipped:
        try:
            fmu_filename = extract(fmu_filename)
        except Exception as e:
            print(f"TEST FAILED - {caller} - zip extraction: {e}")
            return

    try:
        model_description = read_model_description(fmu_filename)

        fmu = fmu_class(
            guid = model_description.guid,
            unzipDirectory = fmu_filename,
            modelIdentifier = model_description.coSimulation.modelIdentifier,
            instanceName='test_instance'
        )

        fmu.instantiate(loggingOn=True)

    except Exception as e:
        print(f"TEST FAILED - {caller} - instantiation: {e}")

        if is_zipped:
            rmtree(fmu_filename, ignore_errors=True)

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

        if is_zipped:
            rmtree(fmu_filename, ignore_errors=True)

        return

    try:
        fmu.terminate()
        fmu.freeInstance()

    except Exception as e:
        print(f"TEST FAILED - {caller} - termination: {e}")

    if is_zipped:
        rmtree(fmu_filename, ignore_errors=True)
    