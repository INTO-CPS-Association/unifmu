#include <stddef.h>
#include <assert.h>
#include <stdio.h>
#include <errno.h>
#include <string.h>
#include <stdbool.h>

#include "fmi2Functions.h"
#include "fmi2FunctionTypes.h"

#ifdef __unix__
#include <dlfcn.h>
#elif defined(_WIN32) || defined(WIN32)
#endif

#define STRINGIFY(x) #x
#define GETSYM(n) fmi2##n##TYPE *fmi2##n = dlsym(handle, STRINGIFY(fmi2##n));

int main(int argc, char **argv)
{

    char *a = STRINGIFY(10);
    char *library_path = argv[1];
    char *uri = argv[2];

    printf("loading library: %s\n", library_path);

    void *handle = dlopen(library_path, RTLD_NOW);

// common functions
#ifdef __unix__
    GETSYM(GetTypesPlatform)
    GETSYM(GetVersion)
    GETSYM(SetDebugLogging)
    GETSYM(Instantiate)
    GETSYM(FreeInstance)
    GETSYM(SetupExperiment)
    GETSYM(EnterInitializationMode)
    GETSYM(ExitInitializationMode)
    GETSYM(Terminate)
    GETSYM(Reset)
    GETSYM(GetReal)
    GETSYM(GetInteger)
    GETSYM(GetBoolean)
    GETSYM(GetString)
    GETSYM(SetReal)
    GETSYM(SetInteger)
    GETSYM(SetBoolean)
    GETSYM(SetString)
    GETSYM(GetFMUstate)
    GETSYM(SetFMUstate)
    GETSYM(FreeFMUstate)
    GETSYM(SerializedFMUstateSize)
    GETSYM(SerializeFMUstate)
    GETSYM(DeSerializeFMUstate)
    GETSYM(GetDirectionalDerivative)
    GETSYM(SetRealInputDerivatives)
    GETSYM(GetRealOutputDerivatives)
    GETSYM(DoStep)
    GETSYM(CancelStep)
    GETSYM(GetStatus)
    GETSYM(GetRealStatus)
    GETSYM(GetIntegerStatus)
    GETSYM(GetBooleanStatus)
    GETSYM(GetStringStatus)
#endif

    if (handle == NULL)
    {
        perror("Failed to open dll");
        return -1;
    }

    double t_start = 0;
    double t_end = 1;
    int steps = 1000;
    double step_size = (t_end - t_start) / steps;

    void *c = fmi2Instantiate("a", fmi2CoSimulation, "", uri, NULL, false, false);

    fmi2SetupExperiment(c, false, 0, t_start, true, t_end);
    fmi2EnterInitializationMode(c);
    fmi2ExitInitializationMode(c);

    // real
    {
        fmi2Real vals[] = {1.0, 1.0};
        fmi2ValueReference refs[] = {0, 1};
        assert(fmi2GetReal(c, refs, 2, vals) == fmi2OK);
        assert(vals[0] == 0 && vals[1] == 0);
        vals[0] = 1.0;
        vals[1] = 1.0;
        assert(fmi2SetReal(c, refs, 2, vals) == fmi2OK);
        refs[0] = 2;
        assert(fmi2GetReal(c, refs, 1, vals) == fmi2OK);
        assert(vals[0] == 2.0);
    }

    // integer
    {
        fmi2Integer vals[] = {1, 1};
        fmi2ValueReference refs[] = {3, 4};
        assert(fmi2GetInteger(c, refs, 2, vals) == fmi2OK);
        assert(vals[0] == 0 && vals[1] == 0);
        vals[0] = 1;
        vals[1] = 1;
        assert(fmi2SetInteger(c, refs, 2, vals) == fmi2OK);
        refs[0] = 5;
        assert(fmi2GetInteger(c, refs, 1, vals) == fmi2OK);
        assert(vals[0] == 2);
    }
    // boolean
    {
        fmi2Boolean vals[] = {true, true};
        fmi2ValueReference refs[] = {6, 7};
        assert(fmi2GetBoolean(c, refs, 2, vals) == fmi2OK);
        assert(vals[0] == false && vals[1] == false);
        vals[0] = true;
        vals[1] = true;
        assert(fmi2SetBoolean(c, refs, 2, vals) == fmi2OK);
        refs[0] = 8;
        assert(fmi2GetBoolean(c, refs, 1, vals) == fmi2OK);
        assert(vals[0] == true);
    }

    // string
    {
        char **vals = NULL;

        fmi2ValueReference refs[] = {9, 10};
        assert(fmi2GetString(c, refs, 2, &vals) == fmi2OK);
        assert(strcmp(vals[0], "") == 0 && strcmp(vals[1], "") == 0);
        vals = malloc(2 * sizeof(char *));
        vals[0] = "abc";
        vals[1] = "def";
        assert(fmi2SetString(c, refs, 2, vals) == fmi2OK);
        free(vals);
        refs[0] = 11;
        assert(fmi2GetString(c, refs, 1, &vals) == fmi2OK);
        assert(strcmp(vals[0], "abcdef") == 0);
    }

    // stepping
    double cur_time = t_start;
    for (int i = 0; i < steps; ++i)
    {

        fmi2DoStep(c, cur_time, step_size, false);
        cur_time += step_size;
    }
    fmi2Terminate(c);
    fmi2FreeInstance(c);
    int res = dlclose(handle);

    return 0;
}