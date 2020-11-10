// we define macros to ease the burden of handling os specific "dlopen" - functionality
#define STRINGIFY(x) #x
#if __unix__
#include <dlfcn.h>
#define CLOSEFUNC dlclose
#define LOADFUNC dlsym
#elif defined(_WIN32) || defined(WIN32)
#include <windows.h>
#include <libloaderapi.h>
#define CLOSEFUNC FreeLibrary
#define LOADFUNC GetProcAddress

#endif
#define IMPORT(n)                                      \
    funcs->n = (n##TYPE *)LOADFUNC(handle, #n);        \
    if (funcs->n == NULL)                              \
    {                                                  \
        printf(STRINGIFY(unable to load function##n)); \
        return -1;                                     \
    }
;

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

int load_library(Fmi2Functions *funcs, const char *filename)
{
#ifdef __unix__
    handle = dlopen(filename, RTLD_NOW);
#elif defined(_WIN32) || defined(WIN32)
    handle = LoadLibrary(filename);
#endif

    IMPORT(fmi2GetTypesPlatform);
    IMPORT(fmi2GetVersion);
    IMPORT(fmi2SetDebugLogging);
    IMPORT(fmi2Instantiate);
    IMPORT(fmi2FreeInstance);
    IMPORT(fmi2SetupExperiment);
    IMPORT(fmi2EnterInitializationMode);
    IMPORT(fmi2ExitInitializationMode);
    IMPORT(fmi2Terminate);
    IMPORT(fmi2Reset);
    IMPORT(fmi2GetReal);
    IMPORT(fmi2GetInteger);
    IMPORT(fmi2GetBoolean);
    IMPORT(fmi2GetString);
    IMPORT(fmi2SetReal);
    IMPORT(fmi2SetInteger);
    IMPORT(fmi2SetBoolean);
    IMPORT(fmi2SetString);
    IMPORT(fmi2GetFMUstate);
    IMPORT(fmi2SetFMUstate);
    IMPORT(fmi2FreeFMUstate);
    IMPORT(fmi2SerializedFMUstateSize);
    IMPORT(fmi2SerializeFMUstate);
    IMPORT(fmi2DeSerializeFMUstate);
    IMPORT(fmi2GetDirectionalDerivative);
    IMPORT(fmi2SetRealInputDerivatives);
    IMPORT(fmi2GetRealOutputDerivatives);
    IMPORT(fmi2DoStep);
    IMPORT(fmi2CancelStep);
    IMPORT(fmi2GetStatus);
    IMPORT(fmi2GetRealStatus);
    IMPORT(fmi2GetIntegerStatus);
    IMPORT(fmi2GetBooleanStatus);
    IMPORT(fmi2GetStringStatus);

    return 0;
}

int free_library()
{
    return CLOSEFUNC(handle);
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