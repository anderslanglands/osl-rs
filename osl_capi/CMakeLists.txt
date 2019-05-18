cmake_minimum_required(VERSION 3.5)
project(osl_capi)

set(CMAKE_CXX_STANDARD 11)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

add_library(osl_capi STATIC osl_capi.cpp)

target_include_directories(
  osl_capi 
  PUBLIC 
    ${OSL_ROOT}/include 
    ${OIIO_ROOT}/include 
    ${OPENEXR_ROOT}/include
)

target_link_libraries(
  osl_capi
  PUBLIC
    ${OSL_ROOT}/lib/liboslexec.so
    ${OIIO_ROOT}/lib/libOpenImageIO.so
    ${OPENEXR_ROOT}/lib/libIex-2_2.so
)

target_compile_options(osl_capi PRIVATE "-Wno-deprecated-register" "-Wno-deprecated")

install(TARGETS osl_capi DESTINATION ${CMAKE_INSTALL_PREFIX})