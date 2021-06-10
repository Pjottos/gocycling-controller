#!/bin/bash

# needs to be run manually for reasons i can't figure out
# it cannot find cassert when run in build.rs
# 

cd ..

echo "#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

pub mod impls;
" > src/binding/mod.rs

bindgen pico-binding/wrapper.hpp \
    --use-core \
    --generate-inline-functions \
    --ctypes-prefix "crate::ctypes" \
    --disable-untagged-union \
    --no-prepend-enum-name \
    --no-layout-tests \
    -- \
    -m32 \
    -I pico-sdk/src/rp2_common/pico_stdio/include \
    -I pico-sdk/src/common/pico_stdlib/include \
    -I pico-sdk/src/common/pico_base/include \
    -I pico-sdk/src/common/pico_time/include \
    -I pico-sdk/src/rp2_common/pico_platform/include \
    -I pico-sdk/src/rp2_common/hardware_base/include \
    -I pico-sdk/src/rp2_common/hardware_timer/include \
    -I pico-sdk/src/rp2_common/hardware_gpio/include \
    -I pico-sdk/src/rp2_common/hardware_uart/include \
    -I pico-sdk/src/rp2_common/hardware_irq/include \
    -I pico-sdk/src/rp2_common/hardware_pwm/include \
    -I pico-sdk/src/rp2_common/hardware_spi/include \
    -I pico-sdk/src/rp2_common/hardware_sync/include \
    -I pico-sdk/src/rp2_common/hardware_rtc/include \
    -I pico-sdk/src/rp2040/hardware_regs/include \
    -I pico-sdk/src/rp2040/hardware_structs/include \
    -I pico-sdk/src/boards/include \
    -I pico-binding/generated \
    >> src/binding/mod.rs

cd build
