# This file is here just for IDE support

cmake_minimum_required(VERSION 3.17)
project(client)

set(CMAKE_CXX_STANDARD 14)

include_directories(kdmapper)

add_executable(
        kdmapper/exports.cpp
        kdmapper/exports.h
        kdmapper/intel_driver.cpp
        kdmapper/intel_driver.hpp
        kdmapper/intel_driver_resource.hpp
        kdmapper/kdmapper.cpp
        kdmapper/kdmapper.hpp
        kdmapper/main.cpp
        kdmapper/nt.hpp
        kdmapper/portable_executable.cpp
        kdmapper/portable_executable.hpp
        kdmapper/service.cpp
        kdmapper/service.hpp
        kdmapper/utils.cpp
        kdmapper/utils.hpp
        )
