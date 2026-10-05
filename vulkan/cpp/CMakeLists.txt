cmake_minimum_required(VERSION 3.20)
project(snake_vk CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Release)
endif()
set(CMAKE_CXX_FLAGS_RELEASE "-O2")

find_package(Vulkan REQUIRED)
find_package(glfw3 3.3 REQUIRED)
find_program(GLSLC glslc REQUIRED)

set(SHADER_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../shaders)
set(SPV_DIR ${CMAKE_BINARY_DIR}/shaders)
file(MAKE_DIRECTORY ${SPV_DIR})
set(SPV_FILES)
foreach(sh quad.vert quad.frag)
  add_custom_command(
    OUTPUT ${SPV_DIR}/${sh}.spv
    COMMAND ${GLSLC} ${SHADER_DIR}/${sh} -o ${SPV_DIR}/${sh}.spv
    DEPENDS ${SHADER_DIR}/${sh}
    COMMENT "glslc ${sh}")
  list(APPEND SPV_FILES ${SPV_DIR}/${sh}.spv)
endforeach()
add_custom_target(shaders DEPENDS ${SPV_FILES})

add_library(snake_game STATIC src/game.cpp)
target_include_directories(snake_game PUBLIC src)
target_compile_options(snake_game PRIVATE -Wall -Wextra)

add_executable(snake_vk src/main.cpp src/renderer.cpp)
target_link_libraries(snake_vk PRIVATE snake_game Vulkan::Vulkan glfw)
target_compile_options(snake_vk PRIVATE -Wall -Wextra -Wno-missing-field-initializers)
target_compile_definitions(snake_vk PRIVATE
  SPV_DIR="${SPV_DIR}"
  $<$<CONFIG:Debug>:SNAKE_VALIDATION=1>)
add_dependencies(snake_vk shaders)

enable_testing()
add_executable(snake_tests tests/snake_tests.cpp)
target_link_libraries(snake_tests PRIVATE snake_game)
target_compile_options(snake_tests PRIVATE -Wall -Wextra)
add_test(NAME snake_tests COMMAND snake_tests)
