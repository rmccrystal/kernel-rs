# This file is here just for IDE support

cmake_minimum_required(VERSION 3.17)
project(client)

set(CMAKE_CXX_STANDARD 14)

add_executable(kdmapper
        intel_driver.cpp
        kdmapper.cpp
        main.cpp
        portable_executable.cpp
        service.cpp
        utils.cpp
        exports.cpp
        )


target_include_directories(kdmapper PRIVATE .)
target_include_directories(kdmapper PUBLIC "C:/Program Files (x86)/Microsoft Visual Studio/2019/Community/VC/Tools/MSVC/14.29.30037/atlmfc/include")
