#include <stdarg.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>

struct log_capture {
    char message[512];
    size_t calls;
    int valid_format;
};

void fmi2_test_logger(const void *environment, const char *instance_name,
                     int status, const char *category, const char *format, ...) {
    struct log_capture *capture = (struct log_capture *)environment;
    (void)instance_name;
    (void)status;
    (void)category;
    capture->calls++;
    capture->valid_format = strcmp(format, "%s") == 0;
    /* Reject the old call convention without reading missing variadic arguments. */
    if (!capture->valid_format) {
        return;
    }

    va_list arguments;
    va_start(arguments, format);
    vsnprintf(capture->message, sizeof(capture->message), format, arguments);
    va_end(arguments);
}
