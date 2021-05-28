add_library(pico_binding
    pico_inline.c
)

target_link_libraries(pico_binding
    pico_stdlib
)
