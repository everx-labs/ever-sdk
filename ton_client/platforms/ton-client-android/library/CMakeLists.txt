cmake_minimum_required(VERSION 3.4.1)
project(tonclientjni VERSION 0.26.0)
include_directories(src/main/include)

# tonclientjni
add_library(
        tonclientjni
        SHARED
        src/main/cpp/tonclientjni.cpp
)

set_target_properties(
        tonclientjni PROPERTIES LINKER_LANGUAGE CXX
)

# tonclient
add_library(
        tonclient
        SHARED
        IMPORTED
)

set_target_properties(
        tonclient PROPERTIES IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/libs/${ANDROID_ABI}/libtonclient.so
        tonclient PROPERTIES IMPORTED_NO_SONAME true
)

target_link_libraries(
        tonclientjni
        tonclient
        android
        log
)
