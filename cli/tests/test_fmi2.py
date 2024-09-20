from fmpy import read_model_description, extract
from fmpy.fmi2 import FMU2Slave
import shutil
import sys


if __name__ == "__main__":
    input_ok = False
    if sys.argv is not None:
        fmu_filename = str(sys.argv[1])


    start_simulation_time = 0.0
    threshold = 2.0
    step_size = 0.01
    # read the model description
    model_description = read_model_description(fmu_filename)

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

    # initialize
    print("Initializing FMI2 test")
    fmu.instantiate()
    fmu.setupExperiment(startTime=start_simulation_time)
    fmu.enterInitializationMode()
    fmu.exitInitializationMode()

    simulation_time = start_simulation_time

    real_c = fmu.getReal([vrs["real_c"]])
    assert real_c == 0.0
    integer_c = fmu.getInteger([vrs["integer_c"]])
    assert integer_c == 0

    fmu.doStep(currentCommunicationPoint=simulation_time, communicationStepSize=step_size)

    fmu.setReal([vrs["real_a"],vrs["real_b"]],[1.0,2.0])
    fmu.setInteger([vrs["integer_a"],vrs["integer_b"]],[1,2])

    fmu.doStep(currentCommunicationPoint=simulation_time, communicationStepSize=step_size)

    real_c = fmu.getReal([vrs["real_c"]])
    assert real_c == 3.0
    integer_c = fmu.getInteger([vrs["integer_c"]])
    assert integer_c == 3
    print("Finalizing FMI2 test")
    fmu.terminate()
    fmu.freeInstance()
    # clean up
    shutil.rmtree(unzipdir, ignore_errors=True)
