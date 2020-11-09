cmake_minimum_required (VERSION 2.8.11)
project (tests)



add_executable(integration_tests tests.c)

target_include_directories(integration_tests PRIVATE .)

if (NOT MSVC)
    target_link_libraries(integration_tests dl)
endif()