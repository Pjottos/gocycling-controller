cmake_minimum_required(VERSION 3.12)

set(ENV{PICO_SDK_PATH} ${CMAKE_SOURCE_DIR}/pico-sdk)

include(pico-sdk/external/pico_sdk_import.cmake)

project(gocycling-controller C CXX ASM)
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

pico_sdk_init()

add_subdirectory(pico-binding)

add_executable(gocycling_controller)

add_custom_target(rust_controller
    COMMAND cargo build --release
)
add_dependencies(rust_controller
    pico_binding
)

add_dependencies(gocycling_controller
    rust_controller
)

target_link_libraries(gocycling_controller
    pico_stdlib
    hardware_pwm
    ${CMAKE_SOURCE_DIR}/target/thumbv6m-none-eabi/release/librust_controller.a
    pico_binding
)

# create map/bin/hex file etc.
pico_add_extra_outputs(gocycling_controller)
