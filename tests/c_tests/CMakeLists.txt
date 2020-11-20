cmake_minimum_required (VERSION 2.8.11)
project (tests)



add_executable(integration_tests tests.c)

target_include_directories(integration_tests PRIVATE .)


if(MSVC)
    target_compile_options(integration_tests PRIVATE /W4 /WX)
else()
    target_link_libraries(integration_tests dl)
    target_compile_options(integration_tests PRIVATE -Wall -Wextra -Werror)
endif()