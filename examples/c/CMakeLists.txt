cmake_minimum_required(VERSION 3.17)
project(c)

set(CMAKE_CXX_STANDARD 14)

link_directories(../../target/release/)
add_executable(c main.cpp tonclient.h)

target_link_libraries(c libton_client.dylib)

