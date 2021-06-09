add_library(pico_binding
    pico_inline.cpp
)

target_link_libraries(pico_binding
    pico_stdlib
    hardware_rtc
    hardware_pwm
)
