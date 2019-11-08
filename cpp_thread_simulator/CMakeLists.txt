cmake_minimum_required(VERSION 3.14)
project(cpp_thread_simulator)

set(CMAKE_CXX_STANDARD 14)

message(STATUS "start running cmake...")

set(BOOST_ROOT /usr/local/Cellar/boost/)
find_package(Boost 1.71.0 COMPONENTS system filesystem REQUIRED)

if(Boost_FOUND)

    message(STATUS "Boost_INCLUDE_DIRS: ${Boost_INCLUDE_DIRS}")
    message(STATUS "Boost_LIBRARIES: ${Boost_LIBRARIES}")
    message(STATUS "Boost_VERSION: ${Boost_VERSION}")

    include_directories(${Boost_INCLUDE_DIRS})

endif()

add_executable(cpp_thread_simulator src/main.cpp)