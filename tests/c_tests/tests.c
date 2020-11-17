#include <stdlib.h>

#if defined(_WIN32) || defined(WIN32)
#include <windows.h>
#include <libloaderapi.h>
#else
#include <dlfcn.h>
#endif

#include <stddef.h>
#include <assert.h>
#include <stdio.h>
#include <errno.h>
#include <string.h>
#include <stdbool.h>

#include "fmi2Functions.h"
#include "fmi2FunctionTypes.h"

typedef struct
{
    fmi2GetTypesPlatformTYPE *fmi2GetTypesPlatform;
    fmi2GetVersionTYPE *fmi2GetVersion;
    fmi2SetDebugLoggingTYPE *fmi2SetDebugLogging;
    fmi2InstantiateTYPE *fmi2Instantiate;
    fmi2FreeInstanceTYPE *fmi2FreeInstance;
    fmi2SetupExperimentTYPE *fmi2SetupExperiment;
    fmi2EnterInitializationModeTYPE *fmi2EnterInitializationMode;
    fmi2ExitInitializationModeTYPE *fmi2ExitInitializationMode;
    fmi2TerminateTYPE *fmi2Terminate;
    fmi2ResetTYPE *fmi2Reset;
    fmi2GetRealTYPE *fmi2GetReal;
    fmi2GetIntegerTYPE *fmi2GetInteger;
    fmi2GetBooleanTYPE *fmi2GetBoolean;
    fmi2GetStringTYPE *fmi2GetString;
    fmi2SetRealTYPE *fmi2SetReal;
    fmi2SetIntegerTYPE *fmi2SetInteger;
    fmi2SetBooleanTYPE *fmi2SetBoolean;
    fmi2SetStringTYPE *fmi2SetString;
    fmi2GetFMUstateTYPE *fmi2GetFMUstate;
    fmi2SetFMUstateTYPE *fmi2SetFMUstate;
    fmi2FreeFMUstateTYPE *fmi2FreeFMUstate;
    fmi2SerializedFMUstateSizeTYPE *fmi2SerializedFMUstateSize;
    fmi2SerializeFMUstateTYPE *fmi2SerializeFMUstate;
    fmi2DeSerializeFMUstateTYPE *fmi2DeSerializeFMUstate;
    fmi2GetDirectionalDerivativeTYPE *fmi2GetDirectionalDerivative;
    fmi2SetRealInputDerivativesTYPE *fmi2SetRealInputDerivatives;
    fmi2GetRealOutputDerivativesTYPE *fmi2GetRealOutputDerivatives;
    fmi2DoStepTYPE *fmi2DoStep;
    fmi2CancelStepTYPE *fmi2CancelStep;
    fmi2GetStatusTYPE *fmi2GetStatus;
    fmi2GetRealStatusTYPE *fmi2GetRealStatus;
    fmi2GetIntegerStatusTYPE *fmi2GetIntegerStatus;
    fmi2GetBooleanStatusTYPE *fmi2GetBooleanStatus;
    fmi2GetStringStatusTYPE *fmi2GetStringStatus;
} Fmi2Functions;

void *handle;

void *load_symbol(const char *name, void *handle)
{
    void *func;
#if defined(_WIN32) || defined(WIN32)
    func = GetProcAddress(handle, name);
#else
    func = dlsym(handle, name);
#endif

    if (func == NULL)
    {
        printf("Unable to load function %s from shared library\n", name);
        abort();
    }
    return func;
}

int load_library(Fmi2Functions *funcs, const char *filename)
{
#if defined(_WIN32) || defined(WIN32)
    handle = LoadLibrary(filename);
#else
    handle = dlopen(filename, RTLD_NOW);
#endif
    funcs->fmi2GetTypesPlatform = (fmi2GetTypesPlatformTYPE *)load_symbol("fmi2GetTypesPlatform", handle);
    funcs->fmi2GetVersion = (fmi2GetVersionTYPE *)load_symbol("fmi2GetVersion", handle);
    funcs->fmi2SetDebugLogging = (fmi2SetDebugLoggingTYPE *)load_symbol("fmi2SetDebugLogging", handle);
    funcs->fmi2Instantiate = (fmi2InstantiateTYPE *)load_symbol("fmi2Instantiate", handle);
    funcs->fmi2FreeInstance = (fmi2FreeInstanceTYPE *)load_symbol("fmi2FreeInstance", handle);
    funcs->fmi2SetupExperiment = (fmi2SetupExperimentTYPE *)load_symbol("fmi2SetupExperiment", handle);
    funcs->fmi2EnterInitializationMode = (fmi2EnterInitializationModeTYPE *)load_symbol("fmi2EnterInitializationMode", handle);
    funcs->fmi2ExitInitializationMode = (fmi2ExitInitializationModeTYPE *)load_symbol("fmi2ExitInitializationMode", handle);
    funcs->fmi2Terminate = (fmi2TerminateTYPE *)load_symbol("fmi2Terminate", handle);
    funcs->fmi2Reset = (fmi2ResetTYPE *)load_symbol("fmi2Reset", handle);
    funcs->fmi2GetReal = (fmi2GetRealTYPE *)load_symbol("fmi2GetReal", handle);
    funcs->fmi2GetInteger = (fmi2GetIntegerTYPE *)load_symbol("fmi2GetInteger", handle);
    funcs->fmi2GetBoolean = (fmi2GetBooleanTYPE *)load_symbol("fmi2GetBoolean", handle);
    funcs->fmi2GetString = (fmi2GetStringTYPE *)load_symbol("fmi2GetString", handle);
    funcs->fmi2SetReal = (fmi2SetRealTYPE *)load_symbol("fmi2SetReal", handle);
    funcs->fmi2SetInteger = (fmi2SetIntegerTYPE *)load_symbol("fmi2SetInteger", handle);
    funcs->fmi2SetBoolean = (fmi2SetBooleanTYPE *)load_symbol("fmi2SetBoolean", handle);
    funcs->fmi2SetString = (fmi2SetStringTYPE *)load_symbol("fmi2SetString", handle);
    funcs->fmi2GetFMUstate = (fmi2GetFMUstateTYPE *)load_symbol("fmi2GetFMUstate", handle);
    funcs->fmi2SetFMUstate = (fmi2SetFMUstateTYPE *)load_symbol("fmi2SetFMUstate", handle);
    funcs->fmi2FreeFMUstate = (fmi2FreeFMUstateTYPE *)load_symbol("fmi2FreeFMUstate", handle);
    funcs->fmi2SerializedFMUstateSize = (fmi2SerializedFMUstateSizeTYPE *)load_symbol("fmi2SerializedFMUstateSize", handle);
    funcs->fmi2SerializeFMUstate = (fmi2SerializeFMUstateTYPE *)load_symbol("fmi2SerializeFMUstate", handle);
    funcs->fmi2DeSerializeFMUstate = (fmi2DeSerializeFMUstateTYPE *)load_symbol("fmi2DeSerializeFMUstate", handle);
    funcs->fmi2GetDirectionalDerivative = (fmi2GetDirectionalDerivativeTYPE *)load_symbol("fmi2GetDirectionalDerivative", handle);
    funcs->fmi2SetRealInputDerivatives = (fmi2SetRealInputDerivativesTYPE *)load_symbol("fmi2SetRealInputDerivatives", handle);
    funcs->fmi2GetRealOutputDerivatives = (fmi2GetRealOutputDerivativesTYPE *)load_symbol("fmi2GetRealOutputDerivatives", handle);
    funcs->fmi2DoStep = (fmi2DoStepTYPE *)load_symbol("fmi2DoStep", handle);
    funcs->fmi2CancelStep = (fmi2CancelStepTYPE *)load_symbol("fmi2CancelStep", handle);
    funcs->fmi2GetStatus = (fmi2GetStatusTYPE *)load_symbol("fmi2GetStatus", handle);
    funcs->fmi2GetRealStatus = (fmi2GetRealStatusTYPE *)load_symbol("fmi2GetRealStatus", handle);
    funcs->fmi2GetIntegerStatus = (fmi2GetIntegerStatusTYPE *)load_symbol("fmi2GetIntegerStatus", handle);
    funcs->fmi2GetBooleanStatus = (fmi2GetBooleanStatusTYPE *)load_symbol("fmi2GetBooleanStatus", handle);
    funcs->fmi2GetStringStatus = (fmi2GetStringStatusTYPE *)load_symbol("fmi2GetStringStatus", handle);

    return 0;
}

int free_library()
{
#if defined(_WIN32) || defined(WIN32)
    FreeLibrary(handle);
#else
    dlclose(handle);
#endif
}

int main(int argc, char **argv)
{

    char *library_path = argv[1];
    char *uri = argv[2];

    printf("loading library: %s\n", library_path);

    Fmi2Functions f;

    assert(load_library(&f, library_path) == 0);

    double t_start = 0;
    double t_end = 1;
    int steps = 1000;
    double step_size = (t_end - t_start) / steps;

    void *c = f.fmi2Instantiate("a", fmi2CoSimulation, "", uri, NULL, false, false);

    f.fmi2SetupExperiment(c, false, 0, t_start, true, t_end);
    f.fmi2EnterInitializationMode(c);
    f.fmi2ExitInitializationMode(c);

    // real
    {
        fmi2Real vals[] = {1.0, 1.0};
        fmi2ValueReference refs[] = {0, 1};
        assert(f.fmi2GetReal(c, refs, 2, vals) == fmi2OK);
        assert(vals[0] == 0 && vals[1] == 0);
        vals[0] = 1.0;
        vals[1] = 1.0;
        assert(f.fmi2SetReal(c, refs, 2, vals) == fmi2OK);
        refs[0] = 2;
        assert(f.fmi2GetReal(c, refs, 1, vals) == fmi2OK);
        assert(vals[0] == 2.0);
    }

    // integer
    {
        fmi2Integer vals[] = {1, 1};
        fmi2ValueReference refs[] = {3, 4};
        assert(f.fmi2GetInteger(c, refs, 2, vals) == fmi2OK);
        assert(vals[0] == 0 && vals[1] == 0);
        vals[0] = 1;
        vals[1] = 1;
        assert(f.fmi2SetInteger(c, refs, 2, vals) == fmi2OK);
        refs[0] = 5;
        assert(f.fmi2GetInteger(c, refs, 1, vals) == fmi2OK);
        assert(vals[0] == 2);
    }

    //boolean
    {
        fmi2Boolean vals[] = {true, true};
        fmi2ValueReference refs[] = {6, 7};
        assert(f.fmi2GetBoolean(c, refs, 2, vals) == fmi2OK);
        assert(vals[0] == false && vals[1] == false);
        vals[0] = true;
        vals[1] = true;
        assert(f.fmi2SetBoolean(c, refs, 2, vals) == fmi2OK);
        refs[0] = 8;
        assert(f.fmi2GetBoolean(c, refs, 1, vals) == fmi2OK);
        assert(vals[0] == true);
    }

    // string
    {
        const char *vals[3];
        fmi2ValueReference refs[] = {9, 10, 11};
        assert(f.fmi2GetString(c, refs, 3, vals) == fmi2OK);
        assert(strcmp(vals[0], "") == 0 && strcmp(vals[1], "") == 0 && strcmp(vals[2], "") == 0);

        vals[0] = "abc";
        vals[1] = "def";
        assert(f.fmi2SetString(c, refs, 2, vals) == fmi2OK);
        assert(f.fmi2GetString(c, refs, 3, vals) == fmi2OK);
        assert(strcmp(vals[2], "abcdef") == 0);
    }

    // stepping
    double cur_time = t_start;
    for (int i = 0; i < steps; ++i)
    {

        f.fmi2DoStep(c, cur_time, step_size, false);
        cur_time += step_size;
    }
    f.fmi2Terminate(c);
    f.fmi2FreeInstance(c);
    free_library();

    return 0;
}