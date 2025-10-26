# cpp/CMakeLists.txt
cmake_minimum_required(VERSION 3.10)
project(nexus_crypto CXX)

set(CMAKE_CXX_STANDARD 17)

find_package(PkgConfig REQUIRED)
pkg_check_modules(SODIUM REQUIRED libsodium)

add_library(nexus_crypto STATIC crypto.cpp)

target_include_directories(nexus_crypto PUBLIC ${SODIUM_INCLUDE_DIRS})
target_link_libraries(nexus_crypto ${SODIUM_LIBRARIES})
