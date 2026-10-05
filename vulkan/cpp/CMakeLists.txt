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
find_package(Threads REQUIRED)
find_program(GLSLC glslc REQUIRED)

# ---- third-party, header-only (pinned)
include(FetchContent)
FetchContent_Declare(miniaudio
  GIT_REPOSITORY https://github.com/mackron/miniaudio.git
  GIT_TAG 0.11.22
  GIT_SHALLOW TRUE
  SOURCE_SUBDIR _no_cmake)
FetchContent_Declare(stb
  GIT_REPOSITORY https://github.com/nothings/stb.git
  GIT_TAG 2c980bb59875b0d32144a71867fbdebb2f77cd20
  SOURCE_SUBDIR _no_cmake)
FetchContent_MakeAvailable(miniaudio stb)

# ---- shaders (shared, compiled at build time)
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

set(ASSET_DIR_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../assets)
set(PUBLIC_ASSET_DIR_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../../public/assets)
cmake_path(NORMAL_PATH ASSET_DIR_PATH)
cmake_path(NORMAL_PATH PUBLIC_ASSET_DIR_PATH)

# ---- third-party implementations, compiled without warnings
add_library(snake_thirdparty STATIC src/thirdparty.cpp)
target_include_directories(snake_thirdparty SYSTEM PUBLIC ${miniaudio_SOURCE_DIR} ${stb_SOURCE_DIR})
target_link_libraries(snake_thirdparty PUBLIC Threads::Threads ${CMAKE_DL_LIBS} m)
target_compile_options(snake_thirdparty PRIVATE -w)

# ---- pure logic: no Vulkan, GLFW or audio
add_library(snake_logic STATIC
  src/color.cpp src/sprites.cpp src/levels.cpp src/player.cpp src/pathfinding.cpp src/robot.cpp
  src/camera.cpp src/race.cpp src/effects.cpp src/progress.cpp src/textlayout.cpp src/collision.cpp
  src/benchstats.cpp)
target_include_directories(snake_logic PUBLIC src)
target_compile_options(snake_logic PRIVATE -Wall -Wextra)

# ---- CPU-side drawing and screens (still no Vulkan / GLFW / audio)
add_library(snake_ui STATIC src/atlas.cpp src/drawlist.cpp src/screens.cpp src/game.cpp)
target_link_libraries(snake_ui PUBLIC snake_logic snake_thirdparty)
target_compile_options(snake_ui PRIVATE -Wall -Wextra)

add_executable(snake_vk src/main.cpp src/renderer.cpp src/audio.cpp)
target_link_libraries(snake_vk PRIVATE snake_ui Vulkan::Vulkan glfw)
target_compile_options(snake_vk PRIVATE -Wall -Wextra -Wno-missing-field-initializers)
target_compile_definitions(snake_vk PRIVATE
  SPV_DIR="${SPV_DIR}"
  ASSET_DIR="${ASSET_DIR_PATH}"
  PUBLIC_ASSET_DIR="${PUBLIC_ASSET_DIR_PATH}"
  $<$<CONFIG:Debug>:SNAKE_VALIDATION=1>)
add_dependencies(snake_vk shaders)

enable_testing()
add_executable(snake_tests tests/snake_tests.cpp)
target_link_libraries(snake_tests PRIVATE snake_ui)
target_compile_options(snake_tests PRIVATE -Wall -Wextra)
target_compile_definitions(snake_tests PRIVATE ASSET_DIR="${ASSET_DIR_PATH}")
add_test(NAME snake_tests COMMAND snake_tests)
